use std::sync::{Arc, Mutex, MutexGuard};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use heapless::String;
use serde::{Deserialize, Serialize};

use super::heapless::HeaplessString;

/// Retrieves and sanitizes a key from nvs.
fn get_key<const N: usize>(nvs: &MutexGuard<'_, EspNvs<NvsDefault>>, key: &str) -> anyhow::Result<String<N>> {
    let mut buf = [0u8; N];

    nvs.get_str(key, &mut buf)?;

    let raw_value = core::str::from_utf8(&buf)
        .map(|s| s.trim_end_matches('0'))
        .unwrap_or("");

    let mut value = HeaplessString::<N>::new();
    value.push_str(raw_value)?;

    Ok(value.clean_string().0)
}

/// Client certificate for mTLS.
#[derive(Deserialize)]
pub struct Certificate {
    #[serde(rename = "cert")]
    pub cert: HeaplessString<1024>,
    #[serde(rename = "certprivkey")]
    pub privkey: HeaplessString<64>,
}

impl Certificate {
    const CERT: &'static str = "CERT";
    const CERT_PRIVKEY: &'static str = "CERTPRIVKEY";

    pub fn is_empty(&self) -> bool {
        self.cert.0.is_empty() || self.privkey.0.is_empty()
    }

    /// Call to set the Certificate configuration in nvs.
    pub fn set_config(&self, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::CERT, self.cert.clean_string().as_str())?;
        nvs.set_str(Self::CERT_PRIVKEY, self.privkey.clean_string().as_str())?;

        Ok(())
    }

    /// Call to retrieve the Certificate configuration from nvs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        // These cannot fail, so we don't care about the unwraps
        Ok(Self {
            cert: HeaplessString(get_key::<1024>(&nvs, Self::CERT).unwrap_or_else(|_| "".try_into().unwrap())),

            privkey: HeaplessString(get_key::<64>(&nvs, Self::CERT_PRIVKEY).unwrap_or_else(|_| "".try_into().unwrap())),
        })
    }
}

/// Stores the wireguard configuration.
#[derive(Serialize, Deserialize, Default)]
pub struct WgConfig {
    #[serde(rename = "address")]
    pub address: HeaplessString<16>,

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
        self.address.0.is_empty()
            || self.port.0.is_empty()
            || self.cli_priv_key.0.is_empty()
            || self.serv_pub_key.0.is_empty()
            || self.allowed_ip.0.is_empty()
            || self.allowed_mask.0.is_empty()
    }

    /// Call to set the Wireguard configuration in nvs.
    pub fn set_config(&self, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::ADDR, self.address.clean_string().as_str())?;
        nvs.set_str(Self::PORT, self.port.clean_string().as_str())?;
        nvs.set_str(Self::CLIENT_PRIV, self.cli_priv_key.clean_string().as_str())?;
        nvs.set_str(Self::SERVER_PUB, self.serv_pub_key.clean_string().as_str())?;

        Ok(())
    }

    /// Call to get an instance of NvsWireguard containing the current stored
    /// Wireguard configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        // These cannot fail, so we don't care about the unwraps
        Ok(Self {
            address: HeaplessString(get_key::<16>(&nvs, Self::ADDR).unwrap_or_else(|_| "".try_into().unwrap())),

            port: HeaplessString(get_key::<8>(&nvs, Self::PORT).unwrap_or_else(|_| "".try_into().unwrap())),

            cli_priv_key: HeaplessString(
                get_key::<64>(&nvs, Self::CLIENT_PRIV).unwrap_or_else(|_| "".try_into().unwrap()),
            ),

            serv_pub_key: HeaplessString(
                get_key::<64>(&nvs, Self::SERVER_PUB).unwrap_or_else(|_| "".try_into().unwrap()),
            ),
            allowed_ip: HeaplessString(
                get_key::<16>(&nvs, Self::ALLOWED_IP).unwrap_or_else(|_| "".try_into().unwrap()),
            ),
            allowed_mask: HeaplessString(
                get_key::<16>(&nvs, Self::ALLOWED_MASK).unwrap_or_else(|_| "".try_into().unwrap()),
            ),
        })
    }
}

/// Stores the WiFi configuration.
#[derive(Deserialize, Default)]
pub struct WifiConfig {
    #[serde(rename = "ssid")]
    pub sta_ssid: HeaplessString<32>,

    #[serde(rename = "passwd")]
    pub sta_passwd: HeaplessString<64>,

    #[serde(rename = "authmethod")]
    pub sta_auth: HeaplessString<32>,
}

impl WifiConfig {
    const STA_AUTH: &'static str = "AUTH";
    const STA_PASSWD: &'static str = "PASSWD";
    const STA_SSID: &'static str = "SSID";

    /// Call to set the wifi configuration in nvs.
    pub fn set_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, config: WifiConfig) -> anyhow::Result<()> {
        let nvs = nvs.lock().unwrap();

        nvs.set_str(Self::STA_SSID, config.sta_ssid.clean_string().as_str())?;
        nvs.set_str(Self::STA_PASSWD, config.sta_passwd.clean_string().as_str())?;
        nvs.set_str(Self::STA_AUTH, config.sta_auth.clean_string().as_str())?;

        Ok(())
    }

    /// Call to get an instance of NvsWifi containing the current stored wifi
    /// configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        // These cannot fail, so we don't care about the unwraps
        Ok(Self {
            sta_ssid: HeaplessString(get_key::<32>(&nvs, Self::STA_SSID).unwrap_or_else(|_| "".try_into().unwrap())),

            sta_passwd: HeaplessString(
                get_key::<64>(&nvs, Self::STA_PASSWD).unwrap_or_else(|_| "".try_into().unwrap()),
            ),

            sta_auth: HeaplessString(
                get_key::<32>(&nvs, Self::STA_AUTH).unwrap_or_else(|_| "wpa2personal".try_into().unwrap()),
            ),
        })
    }
}
