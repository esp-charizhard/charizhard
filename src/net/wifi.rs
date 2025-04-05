use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::ipv4;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys::{ip_event_t_IP_EVENT_STA_GOT_IP, ip_event_t_IP_EVENT_STA_LOST_IP};
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi, WifiDriver};

use crate::utils::nvs::WifiConfig;

/// Initializes the WiFi driver and network interface, but does not start it
/// yet. This will be done when the user calls a scan using the web interface
/// provided by the http server.
pub fn wifi_init(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<Arc<Mutex<EspWifi<'static>>>> {
    log::info!("Installing wifi netif...");

    let wifi_driver = WifiDriver::new(modem, sysloop.clone(), Some(nvs.clone()))?;

    let wifi_netif = EspWifi::wrap_all(
        wifi_driver,
        EspNetif::new_with_conf(&NetifConfiguration {
            flags: 0,
            key: "WIFI_STA_DEF".try_into().unwrap(),
            description: "sta".try_into().unwrap(),
            route_priority: 100,
            ip_configuration: Some(ipv4::Configuration::Client(Default::default())), // DHCP
            stack: NetifStack::Sta,
            custom_mac: None,
            got_ip_event_id: NonZeroU32::new(ip_event_t_IP_EVENT_STA_GOT_IP as _),
            lost_ip_event_id: NonZeroU32::new(ip_event_t_IP_EVENT_STA_LOST_IP as _),
        })?,
    )?;

    log::info!("Installed wifi netif!");

    Ok(Arc::new(Mutex::new(wifi_netif)))
}

/// Stores the given configuration in nvs and sets it.
pub fn wifi_set_config(nvs: Arc<Mutex<EspNvs<NvsDefault>>>, wifi: Arc<Mutex<EspWifi<'static>>>) -> anyhow::Result<()> {
    log::info!("Setting wifi configuration...");

    let mut wifi = wifi.lock().unwrap();

    let config = WifiConfig::get_config(Arc::clone(&nvs))?;

    let wifi_config = Configuration::Client(ClientConfiguration {
        ssid: config.ssid.0,
        password: config.password.0,
        auth_method: config.auth_method.as_str().try_into()?,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;

    log::info!("Wifi configuration set!");

    Ok(())
}

const MAX_CONNECTION_CHECKS: u32 = 20;

/// Connects the WiFi network interface to the configured access point.
/// Care should be taken to always call [`EspWifi::set_configuration`] before
/// this function.
pub fn wifi_connect(wifi: Arc<Mutex<EspWifi<'static>>>) -> anyhow::Result<()> {
    log::info!("Connecting to access point..");

    let mut wifi = wifi.lock().unwrap();

    if !wifi.is_started()? {
        log::info!("Starting wifi..");
        wifi.start()?;
    }

    if wifi.is_connected()? {
        log::error!("Already connected to an access point!");
        return Err(anyhow::anyhow!("Already connected to an access point!"));
    }

    wifi.connect()?;

    for retries in 0..=MAX_CONNECTION_CHECKS {
        if wifi.is_connected()? {
            log::info!("Wifi connection established.");
            break;
        }

        log::info!("Waiting for wifi connection...");
        std::thread::park_timeout(std::time::Duration::from_millis(250));

        if retries == MAX_CONNECTION_CHECKS {
            log::error!("Failed to connect to wifi! Incorrect Credentials?");
            return Err(anyhow::anyhow!("Failed to connect to wifi! Incorrect Credentials?"));
        }
    }

    Ok(())
}

/// Disconnects the WiFi network interface from the access point it is connected
/// to.
pub fn wifi_disconnect(wifi: Arc<Mutex<EspWifi<'static>>>) -> anyhow::Result<()> {
    log::info!("Disconnecting from access point..");

    let mut wifi = wifi.lock().unwrap();

    if !wifi.is_started()? {
        wifi.start()?;
        return Ok(());
    }

    if !wifi.is_connected()? {
        return Ok(());
    }

    wifi.disconnect()?;

    Ok(())
}
