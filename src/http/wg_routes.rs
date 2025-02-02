use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use crate::utils::nvs::WgConfig;
use crate::wireguard as wg;

lazy_static::lazy_static!(
    static ref WG_LOCK: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
);

/// Sets the Wireguard related routes for the http server.
pub fn set_routes(http_server: &mut EspHttpServer<'static>, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    // Handler to connect to a wireguard peer
    http_server.fn_handler("/connect-wg", Method::Post, {
        // This is so fucking stupid but we can't do otherwise
        let nvs = Arc::clone(&nvs);

        move |mut request| {
            {
                let mut locked = WG_LOCK.lock().unwrap();
                if *locked {
                    log::warn!("Wireguard connection already in progress!");

                    let connection = request.connection();

                    connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

                    return Ok::<(), Error>(());
                } else {
                    *locked = true;
                }
            }

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

            let wg_conf: WgConfig = serde_urlencoded::from_str(String::from_utf8(body)?.as_str())?;

            WgConfig::set_config(Arc::clone(&nvs), wg_conf)?;

            // Yeah..
            let nvs = Arc::clone(&nvs);

            thread::spawn(move || {
                let success = wg::sync_systime().is_ok() && wg::start_tunnel(Arc::clone(&nvs)).is_ok();

                if !success {
                    let mut locked = WG_LOCK.lock().unwrap();
                    *locked = false;
                }
            });

            let connection = request.connection();

            connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to disconnect from the wireguard peer
    http_server.fn_handler("/disconnect-wg", Method::Get, move |mut request| {
        {
            let locked = WG_LOCK.lock().unwrap();

            if !*locked {
                log::warn!("No wireguard connection found for disconnection attempt!");

                let connection = request.connection();

                connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

                return Ok::<(), Error>(());
            }
        }

        super::check_ip(&mut request)?;

        thread::spawn(move || {
            if wg::end_tunnel().is_ok() {
                let mut locked = WG_LOCK.lock().unwrap();
                *locked = false;
            }
        });

        let connection = request.connection();

        connection.initiate_response(204, Some("OK"), &[("Content-Type", "text/html")])?;

        Ok::<(), Error>(())
    })?;

    // Handler to get current wireguard status (connected/disconnected)
    http_server.fn_handler("/wg-status", Method::Get, {
        let nvs = Arc::clone(&nvs);

        move |mut request| {
            super::check_ip(&mut request)?;

            let is_connected = wg::ctx::WG_CTX.lock().unwrap().is_set();

            let nvs = WgConfig::get_config(Arc::clone(&nvs))?;

            let svg_status = if is_connected { "connected" } else { "disconnected" };

            let status = if is_connected {
                nvs.address.as_str()
            } else {
                "Disconnected"
            };

            let mut html = format!(
                r###"
                    <div class=svg-status-text-container>
                        <img id="{svg_status}-svg-wg" src="{svg_status}.svg">
                        <div id="wg-status-text">{status}</div>
                    </div>
                "###
            );

            if is_connected {
                html.push_str(
                    r###"
                        <button id="disconnect-wg-button" onclick="disconnectWg()">Disconnect</button>
                    "###,
                );
            }

            let connection = request.connection();

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    Ok(())
}
