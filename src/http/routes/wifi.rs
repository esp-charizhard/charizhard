use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::{AuthMethod, EspWifi};

use crate::net;
use crate::utils::nvs::WifiConfig;

/// Sets the WiFi related routes for the http server.
pub fn set_routes(
    http_server: &mut EspHttpServer<'static>,
    nvs: Arc<Mutex<EspNvs<NvsDefault>>>,
    wifi: Arc<Mutex<EspWifi<'static>>>,
) -> anyhow::Result<()> {
    // Handler to disconnect from wifi
    http_server.fn_handler("/disconnect-wifi", Method::Get, {
        let wifi = Arc::clone(&wifi);

        move |mut request| {
            super::check_ip(&mut request)?;

            net::wifi_disconnect(Arc::clone(&wifi))?;

            let connection = request.connection();

            connection.initiate_response(204, None, &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to connect to wifi
    http_server.fn_handler("/connect-wifi", Method::Post, {
        let wifi = Arc::clone(&wifi);

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

            let wifi_conf: WifiConfig = serde_urlencoded::from_str(String::from_utf8(body)?.as_str())?;

            WifiConfig::set_config(Arc::clone(&nvs), wifi_conf)?;

            let nvs_thread = Arc::clone(&nvs);
            let wifi = Arc::clone(&wifi);

            thread::spawn(move || {
                _ = net::wifi_set_config(Arc::clone(&nvs_thread), Arc::clone(&wifi));
                _ = net::wifi_connect(Arc::clone(&wifi));
            });

            let connection = request.connection();

            connection.initiate_response(204, None, &[("Content-Type", "text/html")])?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to get available wifis
    http_server.fn_handler("/scan-wifi", Method::Get, {
        let wifi = Arc::clone(&wifi);

        move |mut request| {
            super::check_ip(&mut request)?;

            let mut wifi = wifi.lock().unwrap();

            if !wifi.is_started()? {
                wifi.start()?;
            }

            // We only allocate 10 AccessPointInfos worth of memory to prevent stack
            // overflow. This value should be modified if some wifis aren't
            // found upon scanning.
            let mut scanned = wifi.scan_n::<10>()?.0.to_vec();

            // Remove dups
            scanned.sort_by(|a, b| a.ssid.cmp(&b.ssid));
            scanned.dedup_by(|a, b| a.ssid == b.ssid);

            // Sort by desc sig strength (values are negative, with 0db max, -50db avg)
            scanned.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));

            let mut html = String::new();

            for access_point in scanned.iter() {
                let auth_method = match access_point.auth_method {
                    Some(AuthMethod::None) => "/unlocked.svg",
                    _ => "/locked.svg",
                };

                let signal_strength = match access_point.signal_strength {
                    -50..=0 => "/signal-4.svg",
                    -60..=-51 => "/signal-3.svg",
                    -70..=-61 => "/signal-2.svg",
                    _ => "/signal-1.svg",
                };

                let ssid = &access_point.ssid;

                let password_html = if access_point.auth_method != Some(AuthMethod::None) {
                    format!(
                        r###"
                            <label for='passwd-{ssid}'>Password</label>
                            <input type='password' id='passwd-{ssid}' name='passwd' required>
                            <div class='error' id='passwd-{ssid}-error'></div>
                        "###
                    )
                } else {
                    format!(
                        r###"
                            <input type='hidden' id='passwd-{ssid}' name='passwd' value="">
                        "###
                    )
                };

                html.push_str(
                    format!(
                        r###"
                        <div class='wifi' id={ssid} onclick='toggleDropdown(event, this)'>
                            <div class='ssid-block'>    
                                <div class='ssid'>{ssid}</div>
                                <div class='signal-auth-container'>
                                    <div class='auth-method'>
                                        <img src='{auth_method}'>
                                    </div>
                                    <div class='signal-strength'>
                                        <img src='{signal_strength}'>
                                    </div>
                                </div>
                            </div>
                            <div class='wifi-connect'>
                                <form id='connect-form-{ssid}' method='post' action='/connect-wifi'>
                                    <input type='hidden' name='authmethod' value='{}'>
                                    <input type='hidden' name='ssid' value='{ssid}'>
                                    {password_html}
                                    <div class="wifi-error"></div>
                                    <button type='submit'>Connect</button>
                                </form>
                            </div>
                        </div>
                    "###,
                        access_point.auth_method.unwrap_or_default(),
                    )
                    .as_str(),
                );
            }

            let connection = request.connection();

            connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            connection.write(html.as_bytes())?;

            Ok::<(), Error>(())
        }
    })?;

    // Handler to get current wifi status (connected/disconnected)
    http_server.fn_handler("/wifi-status", Method::Get, {
        let wifi = Arc::clone(&wifi);

        move |mut request| {
            super::check_ip(&mut request)?;

            let wifi = wifi.lock().unwrap();

            let is_connected = wifi.is_connected()?;

            let binding = wifi.get_configuration()?;

            let ssid = match binding.as_client_conf_ref() {
                Some(config) => {
                    if is_connected {
                        config.ssid.as_str()
                    } else {
                        "Disconnected"
                    }
                }
                None => "Disconnected",
            };

            let svg_status = match is_connected {
                true => "connected",
                false => "disconnected",
            };

            let mut html = format!(
                r###"
                    <div class=svg-status-text-container>
                        <div class="svg-status-img">
                            <img id="{svg_status}-svg-wifi" src="{svg_status}.svg">
                        </div>
                        <div id="wifi-status-text">{ssid}</div>
                    </div>
                "###
            );

            if is_connected {
                html.push_str(
                    r###"
                        <button id="disconnect-wifi-button" onclick="disconnect()">Disconnect</button>
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
