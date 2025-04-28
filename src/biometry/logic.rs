use core::ptr;
use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys::bmlite::{
    gpio_num_t_GPIO_NUM_12,
    gpio_num_t_GPIO_NUM_14,
    gpio_num_t_GPIO_NUM_15,
    gpio_num_t_GPIO_NUM_2,
    gpio_num_t_GPIO_NUM_35,
    gpio_num_t_GPIO_NUM_4,
    interface_t_SPI_INTERFACE,
    spi_host_device_t_SPI2_HOST,
    MTU,
};

use super::commands::*;
use super::ctx::SENSOR_CTX;
use super::{HcpArg, HcpCom, Params, PinsConfig};
use crate::utils::nvs::Fingerprint;

/// Initializes an SPI configurationa long with an HcpCom struct to communicate
/// with the BM-Lite. This should always be called once and only once at th
/// ebeginning of the
fn init_config() -> anyhow::Result<(*mut Params, *mut PinsConfig, *mut HcpCom)> {
    let pkt_buffer = Box::into_raw(Box::new([0u8; 1024 * 3])) as *mut u8;
    let txrx_buffer = Box::into_raw(Box::new([0u8; MTU as usize])) as *mut u8;

    let chain = Box::into_raw(Box::new(HcpCom {
        write: None,
        read: None,
        phy_rx_timeout: 2000,
        pkt_buffer,
        pkt_size: 0,
        pkt_size_max: 1024 * 3,
        txrx_buffer,
        arg: HcpArg::default(),
        bep_result: 0,
    }));

    let pins = Box::into_raw(Box::new(PinsConfig {
        spi_host: spi_host_device_t_SPI2_HOST,
        cs_n_pin: gpio_num_t_GPIO_NUM_2,
        miso_pin: gpio_num_t_GPIO_NUM_35,
        rst_pin: gpio_num_t_GPIO_NUM_4,
        mosi_pin: gpio_num_t_GPIO_NUM_12,
        irq_pin: gpio_num_t_GPIO_NUM_14,
        spi_clk_pin: gpio_num_t_GPIO_NUM_15,
    }));

    let params = Box::into_raw(Box::new(Params {
        iface: interface_t_SPI_INTERFACE,
        port: ptr::null_mut(),
        baudrate: 5_000_000,
        timeout: 3000,
        pins,
        hcp_comm: chain,
    }));

    Ok((params, pins, chain))
}

/// Initializes the sensor.
pub fn init() -> anyhow::Result<()> {
    log::info!("Initializing Sensor..");

    let mut ctx = SENSOR_CTX.lock().unwrap();

    if ctx.is_set() {
        return Err(anyhow::anyhow!("SENSOR_CTX is already set!"));
    }

    let (params, pins_config, chain) = init_config()?;
    // This needs to be called before any other bmlite interface function. Failure
    // to do this results will invariably result in UB.
    init_sensor(params)?;

    ctx.set(params, pins_config, chain);

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
    deinit_sensor(ctx.params)?;

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
    } else {
        log::warn!("A user was already enrolled.");
    }

    Ok(())
}

/// Verifies whether exactly one user is enrolled.
pub fn is_user_enrolled() -> anyhow::Result<bool> {
    log::info!("Checking if a user is enrolled..");

    let ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        log::warn!("SENSOR_CTX is not set! Cannot check non-existent.");
        return Err(anyhow::anyhow!("SENSOR_CTX is not set!"));
    }

    Ok(template_count(ctx.chain)? == 1)
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

/// Stores the template of an enrolled user into nvs for later matching.
pub fn store_template(nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    log::info!("Saving template to ESP32..");

    let ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        log::warn!("SENSOR_CTX is not set! Cannot check non-existent.");
        return Err(anyhow::anyhow!("SENSOR_CTX is not set!"));
    }

    // Retrieve template from sensor.
    let template = get_template::<2048>(ctx.chain, 1)?;

    // Save into nvs.
    Fingerprint::set_template(template, Arc::clone(&nvs))?;

    Ok(())
}

/// Verifies the template stored in nvs against the one the user authenticated
/// with. If the cosine or normalized similarity between the two vectors dips
/// below 90%, we assume tampering has occurred and throw an error.
pub fn match_template(min_sim: f32, nvs: Arc<Mutex<EspNvs<NvsDefault>>>) -> anyhow::Result<()> {
    log::info!("Verifying template match..");

    let ctx = SENSOR_CTX.lock().unwrap();

    if !ctx.is_set() {
        log::warn!("SENSOR_CTX is not set! Cannot check non-existent.");
        return Err(anyhow::anyhow!("SENSOR_CTX is not set!"));
    }

    let stored_template = &*Fingerprint::get_template(Arc::clone(&nvs))?.template;
    let template = &*get_template::<2048>(ctx.chain, 1)?;

    let norm_sim = normalized_similarity(stored_template, template)?;
    let cos_sim = cosine_similarity(stored_template, template)?;

    log::info!("Normalized similarity: {:.2}%", norm_sim * 100.0);
    log::info!("Cosine similarity: {:.2}%", cos_sim * 100.0);

    if norm_sim < min_sim || cos_sim < min_sim {
        return Err(anyhow::anyhow!("Fingerprint template mismatch! Possible tampering detected."));
    }

    log::info!("Template verification passed!");

    Ok(())
}

/// Calculates the normalized similarity (value closeness).
fn normalized_similarity(a: &[u8; 2048], b: &[u8; 2048]) -> anyhow::Result<f32> {
    let total_diff: u32 = a.iter().zip(b.iter()).map(|(x, y)| x.abs_diff(*y) as u32).sum();

    const MAX_DIFF: u32 = 255 * 2048;
    Ok(1.0 - (total_diff as f32 / MAX_DIFF as f32))
}

/// Calculates the cosine similarity (shape similarity).
fn cosine_similarity(a: &[u8; 2048], b: &[u8; 2048]) -> anyhow::Result<f32> {
    let (dot_product, norm_a_sq, norm_b_sq) = a
        .iter()
        .zip(b.iter())
        .fold((0u32, 0u32, 0u32), |(dot, a2, b2), (&x, &y)| {
            (dot + x as u32 * y as u32, a2 + x as u32 * x as u32, b2 + y as u32 * y as u32)
        });

    let norm_a = (norm_a_sq as f32).sqrt();
    let norm_b = (norm_b_sq as f32).sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return Err(anyhow::anyhow!("Zero vector!"));
    }

    Ok(dot_product as f32 / (norm_a * norm_b))
}
