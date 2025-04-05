use std::ffi::CString;
use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::tls::{self, EspTls, X509};

use crate::utils::nvs::{Certificate, WgConfig};

const HOSTNAME: &str = "charizhard-otp.duckdns.org";

static CA_CERT: &str = include_str!("../certs/letsencrypt.pem");

pub fn fetch_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, email: &str, otp: &str) -> anyhow::Result<()> {
    let mut tls = EspTls::new()?;

    let cert = Certificate::get_config(Arc::clone(&nvs))?;

    if cert.is_empty() {
        log::error!("No certificate to request wireguard configuration with!");
        return Err(anyhow::anyhow!("No certificate to request wireguard configuration with!"));
    }

    tls.connect(HOSTNAME, 443, &tls::Config {
        common_name: Some(HOSTNAME),
        ca_cert: Some(X509::pem(&CString::new(CA_CERT.as_bytes())?)),
        client_cert: Some(X509::pem(&CString::new(cert.cert.as_bytes())?)),
        client_key: Some(X509::pem(&CString::new(cert.privkey.as_bytes())?)),
        alpn_protos: Some(&["http/1.1"]),
        ..Default::default()
    })?;

    let request = format!(
        "GET /otp HTTP/1.1\r\nHost: {}\r\nmail: {}\r\notp: {}\r\nConnection: close\r\n\r\n",
        HOSTNAME, email, otp
    );

    log::info!("REQUEST: \n {}", request);

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

    log::info!("MTLS RESPONSE: {}", response);

    // Split headers and body
    let parts: Vec<&str> = response.split("\r\n\r\n").collect();
    let contents = if parts.len() > 1 { parts[1] } else { "" };

    let wg_conf: WgConfig = serde_urlencoded::from_str(contents)?;

    // Write newly fetched config to nvs.
    wg_conf.set_config(Arc::clone(&nvs))?;

    Ok(())
}
