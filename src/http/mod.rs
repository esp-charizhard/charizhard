use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_svc::http::server::{Configuration as HttpServerConfig, EspHttpConnection, EspHttpServer, Method, Request};
use esp_idf_svc::ipv4::Ipv4Addr;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use serde::Deserialize;

use super::net::ETH_GATEWAY;
use crate::biometry;
use crate::utils::nvs::WgConfig;

mod html;
mod mtls;
mod routes;

use html::{admin_html, index_html, otp_html, status_html};
use routes::{set_admin_routes, set_static_routes, set_wg_routes, set_wifi_routes};

#[derive(Deserialize)]
struct OtpRequest {
    email: String,
    otp: String,
}

/// Checks that the source ip of the request is [`ETH_GATEWAY`] + 1. This
/// function should be called at the beginning of every call to `fn_handler` to
/// prevent security vulnerabilities.
fn check_ip(request: &mut Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    let source_ip = request.connection().raw_connection()?.source_ipv4()?;

    // This IP will be the only one allowed to access the http server once it is
    // up. By default, this is set to the DHCP address allocated to the computer
    // connecting to the esp32.
    if source_ip != Ipv4Addr::from(u32::from(ETH_GATEWAY) + 1) {
        log::warn!("Forbidden ip [{}] tried to connect! Returned 403.", source_ip);
        return Err(Error::msg("Forbidden"));
    }

    Ok(())
}

/// Starts the http server.
pub fn start(
    nvs: Arc<Mutex<EspNvs<NvsDefault>>>,
    wifi: Arc<Mutex<EspWifi<'static>>>,
) -> anyhow::Result<EspHttpServer<'static>> {
    let mut http_server = EspHttpServer::new(&HttpServerConfig {
        http_port: 80,
        stack_size: 16384,
        ..Default::default()
    })?;

    set_static_routes(&mut http_server)?;
    set_wifi_routes(&mut http_server, Arc::clone(&nvs), Arc::clone(&wifi))?;
    set_wg_routes(&mut http_server, Arc::clone(&nvs))?;
    set_admin_routes(&mut http_server, Arc::clone(&nvs))?;

    // Handler to get the main config page
    http_server.fn_handler("/", Method::Get, {
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let html = index_html()?;

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to get the otp verification page
    http_server.fn_handler("/otp", Method::Get, {
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let html = otp_html()?;

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to get the wifi / wireguard status page
    http_server.fn_handler("/status", Method::Get, {
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let html = status_html()?;

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to get the admin config page
    http_server.fn_handler("/admin", Method::Get, {
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let html = admin_html()?;

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to check whether this is the first boot
    http_server.fn_handler("/is-first-boot", Method::Get, {
        let nvs = Arc::clone(&nvs);
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let wg_config = WgConfig::get_config(Arc::clone(&nvs))?;

            match wg_config.is_empty() {
                true => connection.initiate_response(200, Some("true"), &[("Content-Type", "text/html")])?,
                false => connection.initiate_response(200, Some("false"), &[("Content-Type", "text/html")])?,
            }

            Ok::<(), Error>(())
        }
    })?;

    // Handler to fetch a wireguard configuration, given an email/otp combo
    http_server.fn_handler("/verify-otp", Method::Post, {
        let nvs = Arc::clone(&nvs);
        move |mut request| {
            self::check_ip(&mut request)?;

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
                    biometry::enroll_user()?;

                    connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?
                }
                Err(_) => connection.initiate_response(401, Some("KO"), &[("Content-Type", "text/html")])?,
            }

            Ok::<(), Error>(())
        }
    })?;

    Ok(http_server)
}
