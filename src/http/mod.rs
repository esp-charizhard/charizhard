use std::sync::{Arc, Mutex};

use anyhow::Error;
use esp_idf_svc::http::server::{Configuration as HttpServerConfig, EspHttpConnection, EspHttpServer, Method, Request};
use esp_idf_svc::ipv4::Ipv4Addr;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::EspWifi;

use super::net::ETH_GATEWAY;

mod html;
mod mtls;
mod routes;

use html::{admin_html, gen_otp_html, index_html, otp_html, status_html};
use routes::{set_admin_routes, set_otp_routes, set_static_routes, set_wg_routes, set_wifi_routes};

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
        stack_size: 20480,
        max_uri_handlers: 50,
        ..Default::default()
    })?;

    set_static_routes(&mut http_server)?;
    set_wifi_routes(&mut http_server, Arc::clone(&nvs), Arc::clone(&wifi))?;
    set_wg_routes(&mut http_server, Arc::clone(&nvs))?;
    set_admin_routes(&mut http_server, Arc::clone(&nvs))?;
    set_otp_routes(&mut http_server, Arc::clone(&nvs))?;

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

    // Handler to get the admin config page
    http_server.fn_handler("/gen-otp", Method::Get, {
        move |mut request| {
            self::check_ip(&mut request)?;

            let connection = request.connection();

            let html = gen_otp_html()?;

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

    Ok(http_server)
}
