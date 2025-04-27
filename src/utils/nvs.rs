use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::Deserialize;

use super::heapless::HeaplessString;

/// Retrieves and sanitizes a string from nvs.
fn get_str<const N: usize>(nvs: &MutexGuard<'_, EspNvs<NvsDefault>>, key: &str) -> anyhow::Result<HeaplessString<N>> {
    let mut buf = [0u8; N];

    nvs.get_str(key, &mut buf)?;

    let raw_value = core::str::from_utf8(&buf).map(|s| s.trim_end_matches('\0'))?;

    let mut value = HeaplessString::<N>::new();

    value.push_str(raw_value)?;

    Ok(value.clean_string())
}

/// Retrieves and sanitizes a fingerprint from nvs.
fn get_fingerprint<const N: usize>(
    nvs: &MutexGuard<'_, EspNvs<NvsDefault>>,
    key: &str,
) -> anyhow::Result<Box<[u8; N]>> {
    let mut buf = Box::new([0u8; N]);

    nvs.get_blob(key, &mut *buf)?;

    Ok(buf)
}

/// Retrives certificate data from nvs.
fn get_certificate<const N: usize>(
    nvs: &MutexGuard<'_, EspNvs<NvsDefault>>,
    key: &str,
) -> anyhow::Result<HeaplessString<N>> {
    let mut buf = [0u8; N];

    nvs.get_str(key, &mut buf)?;

    let mut value = HeaplessString::<N>::new();

    value.push_str(core::str::from_utf8(&buf)?)?;

    Ok(value)
}

pub struct Fingerprint {
    pub template: Box<[u8; 2048]>,
}

impl Fingerprint {
    const TEMPLATE: &'static str = "TEMPLATE";

    pub fn set_template(template: Box<[u8; 2048]>, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_blob(Self::TEMPLATE, &*template)?;

        Ok(())
    }

    pub fn get_template(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        Ok(Self {
            template: get_fingerprint::<2048>(&nvs, Self::TEMPLATE)?,
        })
    }
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

    /// Checks whether the struct contains a certificate.
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
            cert: get_certificate::<2048>(&nvs, Self::CERT)?,

            privkey: get_certificate::<2048>(&nvs, Self::CERT_PRIVKEY)?,
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
    const EMPTY_CONFIG: &'static str = "EMPTY";
    const PORT: &'static str = "PORT";
    const SERVER_PUB: &'static str = "PUBKEY";

    /// Checks whether nvs contains a configuration.
    pub fn is_empty(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<bool> {
        let nvs = nvs.lock().unwrap();

        match get_str::<8>(&nvs, Self::EMPTY_CONFIG)
            .unwrap_or(HeaplessString::from_str("true")?)
            .as_str()
        {
            "false" => Ok(false),
            "true" => Ok(true),
            _ => Ok(true),
        }
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

        nvs.set_str(Self::EMPTY_CONFIG, "false")?;

        Ok(())
    }

    /// Call to get an instance of NvsWireguard containing the current stored
    /// Wireguard configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        Ok(Self {
            address: get_str::<64>(&nvs, Self::ADDR)?,

            port: get_str::<8>(&nvs, Self::PORT)?,

            cli_priv_key: get_str::<64>(&nvs, Self::CLIENT_PRIV)?,

            serv_pub_key: get_str::<64>(&nvs, Self::SERVER_PUB)?,

            allowed_ip: get_str::<16>(&nvs, Self::ALLOWED_IP)?,

            allowed_mask: get_str::<16>(&nvs, Self::ALLOWED_MASK)?,
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
            ssid: get_str::<32>(&nvs, Self::SSID)?,

            password: get_str::<64>(&nvs, Self::PASSWORD)?,

            auth_method: get_str::<32>(&nvs, Self::AUTH_METHOD)?,
        })
    }
}
