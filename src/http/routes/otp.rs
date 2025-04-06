use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::Deserialize;

use crate::biometry;
use crate::http::mtls;
use crate::utils::nvs::WgConfig;

#[derive(Deserialize)]
struct OtpRequest {
    email: String,
    otp: String,
}

pub fn set_routes(http_server: &mut EspHttpServer<'static>, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    // Handler to check whether this is the first boot
    http_server.fn_handler("/is-first-boot", Method::Get, {
        let nvs = Arc::clone(&nvs);
        move |mut request| {
            super::check_ip(&mut request)?;

            let connection = request.connection();

            let wg_config = WgConfig::get_config(Arc::clone(&nvs))?;

            match wg_config.is_empty() {
                true => connection.initiate_response(204, Some("true"), &[("Content-Type", "text/html")])?,
                false => connection.initiate_response(200, Some("false"), &[("Content-Type", "text/html")])?,
            }

            Ok::<(), Error>(())
        }
    })?;

    // Handler to fetch a wireguard configuration, given an email/otp combo
    http_server.fn_handler("/verify-otp", Method::Post, {
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

            let otp_request: OtpRequest = serde_urlencoded::from_str(String::from_utf8(body)?.as_str())?;

            let connection = request.connection();

            match mtls::fetch_config(Arc::clone(&nvs), &otp_request.email, &otp_request.otp) {
                Ok(_) => {
                    // Now that we authenticated the user, we should force them to enroll their
                    // finger before they can proceed
                    // TODO MOVE THIS TO ANOTHER HANDLER TO TELL THE USER WHEN THEY HAVE TO ENROLL
                    biometry::enroll_user()?;

                    connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?
                }
                Err(_) => connection.initiate_response(401, Some("KO"), &[("Content-Type", "text/html")])?,
            }

            Ok::<(), Error>(())
        }
    })?;

    Ok(())
}
