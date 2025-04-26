use std::sync::{Arc, Mutex};

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use utils::nvs::WgConfig;

/// Handles the BM-Lite sensor module HAL and SPI connection.
mod biometry;
/// Handles the http server and its capabilities.
mod http;
/// Handles wifi and ethernet capabilities.
mod net;
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

    biometry::init()?;

    let is_user_enrolled = biometry::is_user_enrolled()?;
    // ! TODO REFACTOR THIS SO SECRETS ARE NOT IN RAM UNTIL USER HAS BEEN AUTHENTICATED
    let wg_config = WgConfig::get_config(Arc::clone(&nvs_config))?;

    biometry::store_template(Arc::clone(&nvs_config))?;

    match (is_user_enrolled, wg_config.is_empty()) {
        // User enrolled, Empty config.
        // Should not happen but is not problematic.
        (true, true) => {}
        // User enrolled, Set config.
        // We need to check for template tampering while the key was in a powered down state.
        (true, false) => {
            // Authenticate user
            while biometry::check_user().is_err() {}
            // Verify user is legitimate by checking whether their fingerprint has
            // drastically changed since the last successful authentication
            if biometry::verify_template(Arc::clone(&nvs_config)).is_err() {
                log::error!("Similitude check failed! Wiping configuration..");

                // biometry::reset()?;

                // unsafe {
                //     esp_idf_svc::sys::nvs_flash_erase();
                //     esp_idf_svc::sys::esp_restart();
                // }
            } else {
                // If the authentication passes, we store the newly updated template into nvs.
                biometry::store_template(Arc::clone(&nvs_config))?;
            }
        }
        // No user enrolled, No config.
        // We do nothing in this case, the key is in factory state.
        (false, true) => {}
        // No user enrolled (or more than 1), Set config.
        // Tampering has occurred. We wipe the dongle.
        (false, false) => {
            log::error!("Tampering detected! Wiping configuration..");

            biometry::reset()?;

            unsafe {
                esp_idf_svc::sys::nvs_flash_erase();
                esp_idf_svc::sys::esp_restart();
            }
        }
    }

    let eth_netif = net::eth_start(peripherals.pins, peripherals.mac, sysloop.clone())?;
    let wifi_netif = net::wifi_init(peripherals.modem, sysloop.clone(), nvs.clone())?;

    let http_server = http::start(Arc::clone(&nvs_config), Arc::clone(&wifi_netif))?;

    std::thread::park();

    Ok(())
}
