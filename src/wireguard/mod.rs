use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ctx::WG_CTX;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::sys::esp;

use crate::utils::nvs::WgConfig as NvsWgConfig;

/// Handles the management of the global context for the wireguard tunnel.
pub mod ctx;

use core::ptr;

use esp_idf_svc::sys::esp_netif_tcpip_exec;
use esp_idf_svc::sys::wg::{
    esp_wireguard_connect,
    esp_wireguard_disconnect,
    esp_wireguard_init,
    esp_wireguard_set_default,
    esp_wireguardif_peer_is_up,
    wireguard_config_t,
    wireguard_ctx_t,
};

pub type WgConfig = wireguard_config_t;
pub type WgCtx = wireguard_ctx_t;

/// The maximum number of attempts to sync system time before declaring the call
/// to [sync_systime] a failure.
const MAX_SNTP_ATTEMPTS: u32 = 10;

/// The maximum number of attempts to connect to a wireguard peer before
/// declaring the call to [start_tunnel] a failure.
const MAX_WG_ATTEMPTS: u32 = 10;

/// Syncs system time with UTC using [`EspSntp`]. This is necessary to establish
/// a wireguard tunnel. Care should thus be taken to always call this function
/// before attempting to establish a connection with a Wireguard peer.
pub fn sync_systime() -> anyhow::Result<()> {
    let sntp = EspSntp::new_default()?;

    for retries in 0..=MAX_SNTP_ATTEMPTS {
        if sntp.get_sync_status() == SyncStatus::Completed {
            log::info!("Time synchronized successfully.");
            break;
        }
        log::info!("Waiting for time synchronization...");
        std::thread::park_timeout(std::time::Duration::from_secs(1));

        if retries == MAX_SNTP_ATTEMPTS {
            log::error!("Failed to synchronize time! Is internet available?");
            return Err(anyhow::anyhow!("No internet to sync time."));
        }
    }

    Ok(())
}

/// Creates "safe" raw pointers for [`WgCtx`] and
/// [`WgConfig`] by retrieving the set configuration from nvs and
/// wrapping them in [`Box`].
fn create_ctx_conf(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<(*mut WgCtx, *mut WgConfig)> {
    let nvs_conf = NvsWgConfig::get_config(Arc::clone(&nvs))?;

    if nvs_conf.is_empty() {
        return Err(anyhow::anyhow!("Attempted to connect to wireguard with no set config!"));
    }

    let config = Box::new(WgConfig {
        private_key: CString::new(nvs_conf.cli_priv_key.as_str())?.into_raw(),
        listen_port: 51820,
        fw_mark: 0,
        public_key: CString::new(nvs_conf.serv_pub_key.as_str())?.into_raw(),
        preshared_key: ptr::null_mut(),
        allowed_ip: CString::new(nvs_conf.allowed_ip.as_str())?.into_raw(),
        allowed_ip_mask: CString::new(nvs_conf.allowed_mask.as_str())?.into_raw(),
        endpoint: CString::new(nvs_conf.address.as_str())?.into_raw(),
        port: nvs_conf.port.as_str().parse()?,
        persistent_keepalive: 20,
    });

    let ctx = Box::new(WgCtx {
        config: ptr::null_mut(),
        netif: ptr::null_mut(),
        netif_default: ptr::null_mut(),
    });

    Ok((Box::into_raw(ctx), Box::into_raw(config)))
}

/// Establishes a tunnel with the peer defined in the `nvs` configuration.
///
/// This configuration should be set using the [`NvsWgConfig`] struct. Care
/// should be taken to always call this function with the `STA netif` connected
/// to an access point.
///
/// No internet connection will result in the function returning
/// after [`MAX_SNTP_ATTEMPTS`] to sync system time have been expanded. An
/// invalid configuration will result in a cleanup of all allocated ressources
/// after [`MAX_WG_ATTEMPTS`] have been expanded.
///
/// This function sets the [`static@WG_CTX`] global variable. Care should be
/// taken NEVER TO DROP this context as it would unvariably result in undefined
/// behavior or crash the program.
pub fn start_tunnel(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    let mut guard = WG_CTX.lock().unwrap();

    // Check if a tunnel is already in service, otherwise we will get either
    // undefined behavior or a crash. Likely both.
    if guard.is_set() {
        log::error!("Tunnel was already started! Disconnect first.");
        return Ok(());
    }

    let (ctx, config) = create_ctx_conf(nvs)?;

    unsafe {
        log::info!("Initializing wireguard..");

        esp!(esp_wireguard_init(config, ctx))?;

        log::info!("Connecting to peer..");

        // Everything to do with ip shenanigans has to be executed in a tcpip context.
        esp!(esp_netif_tcpip_exec(Some(wg_connect_wrapper), ctx as *mut core::ffi::c_void))?;

        for i in 0..=MAX_WG_ATTEMPTS {
            if i == MAX_WG_ATTEMPTS {
                log::error!("Failed to connect to peer, cleaning up.");

                // While we're not connected yet, this allows us to fail gracefully by
                // deinitializing the entire stack to start from a clean slate the
                // next time we make an attempt to connect to a peer.
                esp!(esp_netif_tcpip_exec(Some(wg_disconnect_wrapper), ctx as *mut core::ffi::c_void))?;

                return Err(anyhow::anyhow!("Failed to connect."));
            }

            match esp!(esp_wireguardif_peer_is_up(ctx)) {
                Ok(_) => {
                    log::info!("Peer is up!");
                    break;
                }
                Err(_) => {
                    log::warn!("Peer is down..");
                    std::thread::park_timeout(Duration::from_millis(1000));
                }
            }
        }

        log::info!("Setting default gateway..");

        esp!(esp_netif_tcpip_exec(
            Some(wg_set_default_wrapper),
            ctx as *mut core::ffi::c_void
        ))?;

        // This keeps ctx and config in scope.
        guard.set(ctx, config);

        Ok(())
    }
}

/// Wrapper for a C function that requires execution in a tcpip context using
/// the [`esp_netif_tcpip_exec`] utility.
unsafe extern "C" fn wg_set_default_wrapper(ctx: *mut core::ffi::c_void) -> i32 {
    esp_wireguard_set_default(ctx as *mut WgCtx)
}

/// Wrapper for a C function that requires execution in a tcpip context using
/// the [`esp_netif_tcpip_exec`] utility.
unsafe extern "C" fn wg_connect_wrapper(ctx: *mut core::ffi::c_void) -> i32 {
    esp_wireguard_connect(ctx as *mut WgCtx)
}

/// Wrapper for a C function that requires execution in a tcpip context using
/// the [`esp_netif_tcpip_exec`] utility.
unsafe extern "C" fn wg_disconnect_wrapper(ctx: *mut core::ffi::c_void) -> i32 {
    esp_wireguard_disconnect(ctx as *mut WgCtx)
}

/// Ends an established tunnel with the peer defined in the `nvs` configuration.
///
/// This function resets the [`static@WG_CTX`] global variable. Care should be
/// taken NEVER TO DROP this context before the execution of this function as it
/// would invariably result in either undefined behavior or crash the program.  
pub fn end_tunnel() -> anyhow::Result<()> {
    log::info!("Disconnecting from peer..");

    let mut guard = WG_CTX.lock().unwrap();

    if !guard.is_set() {
        log::error!("Attempted to disconnect without prior connection!");
        return Ok(());
    }

    unsafe {
        esp!(esp_netif_tcpip_exec(
            Some(wg_disconnect_wrapper),
            guard.0 as *mut core::ffi::c_void
        ))?;
    }

    guard.reset();

    Ok(())
}
