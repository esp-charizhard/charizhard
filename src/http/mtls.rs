use core::ffi::CStr;
use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::tls::{self, EspTls, X509};

use crate::utils::nvs::{Certificate, WgConfig};

const CA_CERT: &str = include_str!("../certs/letsencrypt.pem");

const HOSTNAME: &str = "charizhard.duckdns.org";

pub fn fetch_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, email: &str, otp: &str) -> anyhow::Result<()> {
    let mut tls = EspTls::new()?;

    // need to null-terminate the cert
    let ca_cert = [CA_CERT.as_bytes(), &[0]].concat();

    let client_cert = Certificate::get_config(Arc::clone(&nvs))?;

    if client_cert.is_empty() {
        log::error!("No certificate to request wireguard configuration with!");
        return Err(anyhow::anyhow!("No certificate to request wireguard configuration with!"));
    }

    tls.connect(HOSTNAME, 443, &tls::Config {
        common_name: Some(HOSTNAME),
        ca_cert: Some(X509::pem(CStr::from_bytes_with_nul(&ca_cert)?)),
        client_cert: Some(X509::pem(CStr::from_bytes_with_nul(client_cert.cert.as_bytes())?)),
        client_key: Some(X509::pem(CStr::from_bytes_with_nul(client_cert.privkey.as_bytes())?)),
        alpn_protos: Some(&["http/1.1"]),
        ..Default::default()
    })?;

    let body_data = serde_urlencoded::to_string([("email", email), ("otp", otp)])?;

    let request = format!(
        "POST /get-config HTTP/1.1\r\nHost: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: \
         {}\r\nConnection: close\r\n\r\n{}",
        HOSTNAME,
        body_data.len(),
        body_data
    );

    tls.write_all(request.as_bytes())?;

    let mut body = Vec::new();
    let mut buffer = [0u8; 128];

    loop {
        match tls.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&buffer[..n]),
            Err(e) => return Err(e.into()),
        }
    }

    let response = String::from_utf8(body)?;

    // Split headers and body
    let parts: Vec<&str> = response.split("\r\n\r\n").collect();
    let contents = if parts.len() > 1 { parts[1] } else { "" };

    let wg_conf: WgConfig = serde_urlencoded::from_str(contents)?;

    // Write newly fetched config to nvs.
    wg_conf.set_config(Arc::clone(&nvs))?;

    Ok(())
}
