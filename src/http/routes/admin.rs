use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use crate::biometry;
use crate::utils::nvs::Certificate;

pub fn set_routes(http_server: &mut EspHttpServer<'static>, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    http_server.fn_handler("/reset-config", Method::Get, {
        move |mut request| {
            super::check_ip(&mut request)?;

            let connection = request.connection();

            // Remove all templates from the sensor
            biometry::reset()?;

            unsafe {
                esp_idf_svc::sys::nvs_flash_erase();
                esp_idf_svc::sys::nvs_flash_init();
            }

            connection.initiate_response(204, None, &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    http_server.fn_handler("/set-config", Method::Post, {
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

            connection.initiate_response(204, None, &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    Ok(())
}
