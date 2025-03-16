use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::EspWifi;

use crate::utils::nvs::WgConfig;
use crate::{biometry, wireguard as wg};

lazy_static::lazy_static!(
    static ref WG_LOCK: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
);

/// Sets the Wireguard related routes for the http server.
pub fn set_routes(
    http_server: &mut EspHttpServer<'static>,
    nvs: Arc<Mutex<EspNvs<NvsDefault>>>,
    wifi: Arc<Mutex<EspWifi<'static>>>,
) -> anyhow::Result<()> {
    // Handler to connect to a wireguard peer
    http_server.fn_handler("/connect-wg", Method::Get, {
        let nvs = Arc::clone(&nvs);
        let wifi = Arc::clone(&wifi);

        move |mut request| {
            super::check_ip(&mut request)?;

            let connection = request.connection();

            // Hang until user authenticates their finger
            if biometry::check_user().is_err() {
                connection.initiate_response(401, Some("Bad Fingerprint"), &[("Content-Type", "text/html")])?;

                return Ok::<(), Error>(());
            }

            {
                let wifi = wifi.lock().unwrap();
                if !wifi.is_connected()? {
                    log::warn!("Cannot initiate wireguard tunnel without wifi connection being established.");
                    connection.initiate_response(412, Some("Wifi Disconected"), &[("Content-Type", "text/html")])?;

                    return Ok::<(), Error>(());
                }
            }

            {
                let mut locked = WG_LOCK.lock().unwrap();
                if *locked {
                    log::warn!("Wireguard connection already in progress!");

                    connection.initiate_response(208, Some("Already Connected"), &[("Content-Type", "text/html")])?;

                    return Ok::<(), Error>(());
                } else {
                    *locked = true;
                }
            }

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
                        <div class="svg-status-img">
                            <img id="{svg_status}-svg-wg" src="{svg_status}.svg">
                        </div>
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
