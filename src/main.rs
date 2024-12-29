use std::sync::{Arc, Mutex};

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use network::{eth, wifi};

/// Handles the http server and its capabilities.
mod http;
/// Handles wifi and ethernet capabilities.
mod network;
/// Handles over-the-air updates.
mod ota;
/// Handles non-volatile storage.
mod utils;
/// Handles wireguard tunnel creation and destruction.
mod wireguard;

/// Runs the main sysloop.
#[allow(unused_variables)]
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let nvs_config = Arc::new(Mutex::new(EspNvs::new(nvs.clone(), "config", true)?));

    let eth_netif = eth::start(peripherals.pins, peripherals.mac, sysloop.clone())?;
    let wifi_netif = wifi::init(peripherals.modem, sysloop.clone(), nvs.clone())?;

    let http_server = http::start(Arc::clone(&nvs_config), Arc::clone(&wifi_netif))?;

    std::thread::park();

    Ok(())
}
