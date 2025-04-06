use std::sync::{Arc, Mutex, MutexGuard};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::Deserialize;

use super::heapless::HeaplessString;

/// Retrieves and sanitizes a key from nvs.
fn get_key<const N: usize>(nvs: &MutexGuard<'_, EspNvs<NvsDefault>>, key: &str) -> anyhow::Result<HeaplessString<N>> {
    let mut buf = [0u8; N];

    nvs.get_str(key, &mut buf)?;

    let raw_value = core::str::from_utf8(&buf)
        .map(|s| s.trim_end_matches('\0'))
        .unwrap_or("");

    let mut value = HeaplessString::<N>::new();
    value.push_str(raw_value)?;

    Ok(value.clean_string())
}

/// Client certificate data for mTLS.
#[derive(Deserialize)]
pub struct Certificate {
    #[serde(rename = "cert")]
    pub cert: HeaplessString<2048>,
    #[serde(rename = "certprivkey")]
    pub privkey: HeaplessString<2048>,
}

impl Certificate {
    const CERT: &'static str = "CERT";
    const CERT_PRIVKEY: &'static str = "CERTPRIVKEY";

    fn get_key<const N: usize>(
        nvs: &MutexGuard<'_, EspNvs<NvsDefault>>,
        key: &str,
    ) -> anyhow::Result<HeaplessString<N>> {
        let mut buf = [0u8; N];

        nvs.get_str(key, &mut buf)?;

        let raw_value = core::str::from_utf8(&buf).unwrap_or("");

        let mut value = HeaplessString::<N>::new();
        value.push_str(raw_value)?;

        Ok(value)
    }

    pub fn is_empty(&self) -> bool {
        self.cert.is_empty() || self.privkey.is_empty()
    }

    /// Call to set the Certificate configuration in nvs.
    pub fn set_config(&self, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::CERT, self.cert.as_str())?;
        nvs.set_str(Self::CERT_PRIVKEY, self.privkey.as_str())?;

        Ok(())
    }

    /// Call to retrieve the Certificate configuration from nvs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        Ok(Self {
            cert: Certificate::get_key::<2048>(&nvs, Self::CERT).unwrap_or_else(|_| HeaplessString::new()),

            privkey: Certificate::get_key::<2048>(&nvs, Self::CERT_PRIVKEY).unwrap_or_else(|_| HeaplessString::new()),
        })
    }
}

/// Stores the wireguard configuration.
#[derive(Deserialize)]
pub struct WgConfig {
    #[serde(rename = "address")]
    pub address: HeaplessString<64>,

    #[serde(rename = "port")]
    pub port: HeaplessString<8>,

    #[serde(rename = "privkey")]
    pub cli_priv_key: HeaplessString<64>,

    #[serde(rename = "pubkey")]
    pub serv_pub_key: HeaplessString<64>,

    #[serde(rename = "allowedip")]
    pub allowed_ip: HeaplessString<16>,

    #[serde(rename = "allowedmask")]
    pub allowed_mask: HeaplessString<16>,
}

impl WgConfig {
    const ADDR: &'static str = "ADDR";
    const ALLOWED_IP: &'static str = "ALLOWEDIP";
    const ALLOWED_MASK: &'static str = "ALLOWEDMASK";
    const CLIENT_PRIV: &'static str = "PRIVKEY";
    const PORT: &'static str = "PORT";
    const SERVER_PUB: &'static str = "PUBKEY";

    pub fn is_empty(&self) -> bool {
        self.address.is_empty()
            || self.port.is_empty()
            || self.cli_priv_key.is_empty()
            || self.serv_pub_key.is_empty()
            || self.allowed_ip.is_empty()
            || self.allowed_mask.is_empty()
    }

    /// Call to set the Wireguard configuration in nvs.
    pub fn set_config(&self, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::ADDR, self.address.clean_string().as_str())?;
        nvs.set_str(Self::PORT, self.port.clean_string().as_str())?;
        nvs.set_str(Self::CLIENT_PRIV, self.cli_priv_key.clean_string().as_str())?;
        nvs.set_str(Self::SERVER_PUB, self.serv_pub_key.clean_string().as_str())?;
        nvs.set_str(Self::ALLOWED_IP, self.allowed_ip.clean_string().as_str())?;
        nvs.set_str(Self::ALLOWED_MASK, self.allowed_mask.clean_string().as_str())?;

        Ok(())
    }

    /// Call to get an instance of NvsWireguard containing the current stored
    /// Wireguard configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        Ok(Self {
            address: get_key::<64>(&nvs, Self::ADDR).unwrap_or_else(|_| HeaplessString::new()),

            port: get_key::<8>(&nvs, Self::PORT).unwrap_or_else(|_| HeaplessString::new()),

            cli_priv_key: get_key::<64>(&nvs, Self::CLIENT_PRIV).unwrap_or_else(|_| HeaplessString::new()),

            serv_pub_key: get_key::<64>(&nvs, Self::SERVER_PUB).unwrap_or_else(|_| HeaplessString::new()),

            allowed_ip: get_key::<16>(&nvs, Self::ALLOWED_IP).unwrap_or_else(|_| HeaplessString::new()),

            allowed_mask: get_key::<16>(&nvs, Self::ALLOWED_MASK).unwrap_or_else(|_| HeaplessString::new()),
        })
    }
}

/// Stores the WiFi configuration.
#[derive(Deserialize)]
pub struct WifiConfig {
    #[serde(rename = "ssid")]
    pub ssid: HeaplessString<32>,

    #[serde(rename = "passwd")]
    pub password: HeaplessString<64>,

    #[serde(rename = "authmethod")]
    pub auth_method: HeaplessString<32>,
}

impl WifiConfig {
    const AUTH_METHOD: &'static str = "AUTH";
    const PASSWORD: &'static str = "PASSWD";
    const SSID: &'static str = "SSID";

    /// Call to set the wifi configuration in nvs.
    pub fn set_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, config: WifiConfig) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::SSID, config.ssid.clean_string().as_str())?;
        nvs.set_str(Self::PASSWORD, config.password.clean_string().as_str())?;
        nvs.set_str(Self::AUTH_METHOD, config.auth_method.clean_string().as_str())?;

        Ok(())
    }

    /// Call to get an instance of NvsWifi containing the current stored wifi
    /// configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        Ok(Self {
            ssid: get_key::<32>(&nvs, Self::SSID).unwrap_or_else(|_| HeaplessString::new()),

            password: get_key::<64>(&nvs, Self::PASSWORD).unwrap_or_else(|_| HeaplessString::new()),

            auth_method: get_key::<32>(&nvs, Self::AUTH_METHOD).unwrap_or_else(|_| HeaplessString::new()),
        })
    }
}
