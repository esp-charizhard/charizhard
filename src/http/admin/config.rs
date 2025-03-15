use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use crate::biometry;
use crate::utils::heapless::HeaplessString;
use crate::utils::nvs::{Certificate, WgConfig};

pub fn set_routes(http_server: &mut EspHttpServer<'static>, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    http_server.fn_handler("/reset-config", Method::Get, {
        let nvs = Arc::clone(&nvs);

        move |mut request| {
            super::check_ip(&mut request)?;

            let connection = request.connection();

            // Overwrite the client's private key in nvs with 0s for a clean wipe.
            let wiped_conf = WgConfig {
                address: HeaplessString::<16>::from_str(&"\0".repeat(16))?,
                port: HeaplessString::<8>::from_str(&"\0".repeat(8))?,
                cli_priv_key: HeaplessString::<64>::from_str(&"\0".repeat(64))?,
                serv_pub_key: HeaplessString::<64>::from_str(&"\0".repeat(64))?,
                allowed_ip: HeaplessString::<16>::from_str(&"\0".repeat(16))?,
                allowed_mask: HeaplessString::<16>::from_str(&"\0".repeat(16))?,
            };

            wiped_conf.set_config(Arc::clone(&nvs))?;

            // Remove all templates from the sensor
            biometry::reset()?;

            // Remove certificate and private key from nvs.
            let wiped_conf = Certificate {
                cert: HeaplessString::<1024>::from_str(&"\0".repeat(1024))?,
                privkey: HeaplessString::<64>::from_str(&"\0".repeat(64))?,
            };

            wiped_conf.set_config(Arc::clone(&nvs))?;

            connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    http_server.fn_handler("/enroll-user", Method::Get, {
        move |mut request| {
            super::check_ip(&mut request)?;

            let connection = request.connection();

            biometry::enroll_user()?;

            connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    http_server.fn_handler("/set-cert", Method::Post, {
        let nvs = Arc::clone(&nvs);

        move |mut request| {
            super::check_ip(&mut request)?;

            let mut body = Vec::new();
            let mut buffer = [0u8; 128];

            loop {
                match request.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => body.extend_from_slice(&buffer[..n]),
                    Err(e) => return Err(e.into()),
                }
            }

            let cert_conf: Certificate = serde_urlencoded::from_str(String::from_utf8(body)?.as_str())?;

            cert_conf.set_config(Arc::clone(&nvs))?;

            let connection = request.connection();

            connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    Ok(())
}
