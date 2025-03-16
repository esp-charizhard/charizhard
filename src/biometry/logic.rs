use core::ptr;

use esp_idf_svc::sys::bmlite::{interface_t_SPI_INTERFACE, MTU};

use super::commands::*;
use super::ctx::SENSOR_CTX;
use super::{HcpArg, HcpCom, Params};

/// Initializes an SPI configurationa long with an HcpCom struct to communicate
/// with the BM-Lite. This should always be called once and only once at th
/// ebeginning of the
fn init_config() -> anyhow::Result<(*mut Params, *mut HcpCom)> {
    let pkt_buffer = Box::into_raw(Box::new([0u8; 1024 * 5])) as *mut u8;
    let txrx_buffer = Box::into_raw(Box::new([0u8; MTU as usize])) as *mut u8;

    let chain = Box::into_raw(Box::new(HcpCom {
        write: None,
        read: None,
        phy_rx_timeout: 2000,
        pkt_buffer,
        pkt_size: 0,
        pkt_size_max: 1024 * 5,
        txrx_buffer,
        arg: HcpArg::default(),
        bep_result: 0,
    }));

    let params = Box::into_raw(Box::new(Params {
        iface: interface_t_SPI_INTERFACE,
        port: ptr::null_mut(),
        baudrate: 5_000_000,
        timeout: 3000,
        hcp_comm: chain,
    }));

    Ok((params, chain))
}

/// Initializes the sensor.
pub fn init() -> anyhow::Result<()> {
    log::info!("Initializing Sensor..");

    let mut ctx = SENSOR_CTX.lock().unwrap();

    if ctx.is_set() {
        return Err(anyhow::anyhow!("SENSOR_CTX is already set!"));
    }

    let (params, chain) = init_config()?;

    // This needs to be called before any other bmlite interface function. Failure
    // to do this results will invariably result in UB.
    init_sensor(params)?;

    ctx.set(params, chain);

    Ok(())
}

/// Removes all secrets from flash memory and reboots the sensor.
/// Care should be taken to call this function AFTER a finger has already been
/// enrolled. Failure to do this will invariably result in UB.
pub fn reset() -> anyhow::Result<()> {
    log::info!("Resetting sensor..");

    let mut ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        return Ok(());
    }

    reset_sensor_calibration(ctx.chain)?;
    remove_all_templates(ctx.chain)?;
    hardware_reset(ctx.chain)?;

    ctx.reset();

    Ok(())
}

/// Enrolls a a new user. This should only ever need to be done once per key
/// life, or when the key has been reset to factory settings.
pub fn enroll_user() -> anyhow::Result<()> {
    log::info!("Enrolling user..");

    let mut ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        log::warn!("SENSOR_CTX was not set. Initializing..");

        // init needs the lock
        drop(ctx);
        init()?;

        // retake
        ctx = SENSOR_CTX.lock().unwrap();
    }

    // Verify no other user is already enrolled
    if template_count(ctx.chain)? == 0 {
        calibrate_sensor(ctx.chain)?;
        software_reset(ctx.chain)?;
        enroll_finger(ctx.chain)?;
        save_template(ctx.chain, 1)?;
    }
    else {
        log::warn!("A user was already enrolled.");
    }

    Ok(())
}

/// Checks whether a finger is recognized.
/// Care should be taken to call this function AFTER a finger has already been
/// enrolled. Failure to do this will invariably result in UB.
pub fn check_user() -> anyhow::Result<()> {
    log::info!("Checking finger..");

    let ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        log::warn!("SENSOR_CTX is not set! Cannot check non-existent.");
        return Err(anyhow::anyhow!("SENSOR_CTX is not set!"));
    }

    match identify_finger(ctx.chain, 3000, 1) {
        Ok(true) => {
            log::info!("Finger OK.");
            Ok(())
        }
        Ok(false) => {
            log::warn!("Finger KO.");
            Err(anyhow::anyhow!("Finger KO."))
        }
        Err(_) => Err(anyhow::anyhow!("Failed to identify finger!")),
    }
}
