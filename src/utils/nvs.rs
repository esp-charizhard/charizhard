use std::sync::{Arc, Mutex, MutexGuard};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use heapless::String;
use serde::{Deserialize, Serialize};

use super::heapless::HeaplessString;

/// Stores the wireguard configuration.
#[derive(Serialize, Deserialize, Default)]
pub struct WgConfig {
    #[serde(rename = "address")]
    pub address: HeaplessString<32>,

    #[serde(rename = "port")]
    pub port: HeaplessString<16>,

    #[serde(rename = "privkey")]
    pub cli_priv_key: HeaplessString<64>,

    #[serde(rename = "pubkey")]
    pub serv_pub_key: HeaplessString<64>,

    #[serde(rename = "rember", default)]
    pub remember_me: HeaplessString<8>,
}

impl WgConfig {
    const ADDR: &'static str = "ADDR";
    const CLIENT_PRIV: &'static str = "PRIVKEY";
    const DEFAULT_ADDR: &str = "";
    const DEFAULT_CLIENT_PRIV: &str = "";
    const DEFAULT_PORT: &str = "51820";
    const DEFAULT_REMEMBER_ME: &'static str = "false";
    const DEFAULT_SERVER_PUB: &str = "";
    const PORT: &'static str = "PORT";
    const REMEMBER_ME: &'static str = "REMBER";
    const SERVER_PUB: &'static str = "PUBKEY";

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

    /// Call to set the Wireguard configuration in nvs.
    pub fn set_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, config: WgConfig) -> anyhow::Result<()> {
        let mut nvs = nvs.lock().unwrap();

        nvs.set_str(Self::ADDR, config.address.clean_string().as_str())?;
        nvs.set_str(Self::PORT, config.port.clean_string().as_str())?;
        nvs.set_str(Self::CLIENT_PRIV, config.cli_priv_key.clean_string().as_str())?;
        nvs.set_str(Self::SERVER_PUB, config.serv_pub_key.clean_string().as_str())?;

        if config.remember_me.as_str() == "on" {
            nvs.set_str(Self::REMEMBER_ME, "true")?;
        } else {
            nvs.set_str(Self::REMEMBER_ME, "false")?;
        }

        Ok(())
    }

    /// Call to get an instance of NvsWireguard containing the current stored
    /// Wireguard configs.
    pub fn get_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<Self> {
        let nvs = nvs.lock().unwrap();

        // These cannot fail, so we don't care about the unwraps
        Ok(Self {
            address: HeaplessString(
                WgConfig::get_key::<32>(&nvs, Self::ADDR).unwrap_or_else(|_| Self::DEFAULT_ADDR.try_into().unwrap()),
            )
            .clean_string(),

            port: HeaplessString(
                WgConfig::get_key::<16>(&nvs, Self::PORT).unwrap_or_else(|_| Self::DEFAULT_PORT.try_into().unwrap()),
            )
            .clean_string(),

            cli_priv_key: HeaplessString(
                WgConfig::get_key::<64>(&nvs, Self::CLIENT_PRIV)
                    .unwrap_or_else(|_| Self::DEFAULT_CLIENT_PRIV.try_into().unwrap()),
            )
            .clean_string(),

            serv_pub_key: HeaplessString(
                WgConfig::get_key::<64>(&nvs, Self::SERVER_PUB)
                    .unwrap_or_else(|_| Self::DEFAULT_SERVER_PUB.try_into().unwrap()),
            )
            .clean_string(),

            remember_me: HeaplessString(
                WgConfig::get_key::<8>(&nvs, Self::REMEMBER_ME)
                    .unwrap_or_else(|_| Self::DEFAULT_REMEMBER_ME.try_into().unwrap()),
            )
            .clean_string(),
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
    const DEFAULT_STA_AUTH: &str = "wpa2personal";
    const DEFAULT_STA_PASSWD: &str = "";
    const DEFAULT_STA_SSID: &str = "";
    const STA_AUTH: &'static str = "AUTH";
    const STA_PASSWD: &'static str = "PASSWD";
    const STA_SSID: &'static str = "SSID";

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

    /// Call to set the wifi configuration in nvs.
    pub fn set_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, config: WifiConfig) -> anyhow::Result<()> {
        let mut nvs = nvs.lock().unwrap();

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
            sta_ssid: HeaplessString(
                WifiConfig::get_key::<32>(&nvs, Self::STA_SSID)
                    .unwrap_or_else(|_| Self::DEFAULT_STA_SSID.try_into().unwrap()),
            )
            .clean_string(),

            sta_passwd: HeaplessString(
                WifiConfig::get_key::<64>(&nvs, Self::STA_PASSWD)
                    .unwrap_or_else(|_| Self::DEFAULT_STA_PASSWD.try_into().unwrap()),
            )
            .clean_string(),

            sta_auth: HeaplessString(
                WifiConfig::get_key::<32>(&nvs, Self::STA_AUTH)
                    .unwrap_or_else(|_| Self::DEFAULT_STA_AUTH.try_into().unwrap()),
            )
            .clean_string(),
        })
    }
}
