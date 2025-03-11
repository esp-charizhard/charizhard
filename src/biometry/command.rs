#![allow(unused)]

use std::ops::Add;

use esp_idf_svc::sys::bmlite::platform_init;

use super::functions::*;
use super::results::{debug, FPC_OK, GENERAL_ERROR};
use super::{HcpCom, Params};

/// Initializes the sensor. This function should be called once and only once
/// per program, at the beginning of the mainloop.
pub fn init_sensor(params: *mut Params) -> anyhow::Result<()> {
    unsafe {
        let result = platform_init(params as *mut _);
        (result == FPC_OK)
            .then(|| log::info!("init_sensor: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Retrieves the version of the firmware running on the BM-Lite sensor.
pub fn version(chain: *mut HcpCom) -> anyhow::Result<String> {
    unsafe {
        let mut version = [0u8; 128];

        let result = bep_version(chain, version.as_mut_ptr() as *mut _, version.len() as _);
        (result == FPC_OK)
            .then(|| log::info!("version: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(String::from_utf8(version.to_vec())?)
    }
}

/// Calibrates the sensor. A reboot is required for this to take effect.
pub fn calibrate_sensor(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sensor_calibrate(chain);
        (result == FPC_OK)
            .then(|| log::info!("calibrate_sensor: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Resets sensor calibration. A reboot is required for this to take effect.
pub fn reset_sensor_calibration(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sensor_calibrate_remove(chain);
        (result == FPC_OK)
            .then(|| log::info!("reset_sensor_calibration: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Performs a hardware reset of the fingerprint sensor via the RST pin.
/// Equivalent to a reboot. No data in flash is lost. All data in ram is lost.
pub fn hardware_reset(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sensor_reset(chain);
        (result == FPC_OK)
            .then(|| log::info!("hw_reset: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Performs a software reset of the fignerprint sensor. Equivalent to a reboot.
/// No data in flash is lost. All data in ram is lost.
pub fn software_reset(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sw_reset(chain);
        (result == FPC_OK)
            .then(|| log::info!("sw_reset: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Removes a specific template from flash storage, given an id.
pub fn remove_template(chain: *mut HcpCom, template_id: u16) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_remove(chain, template_id);
        (result == FPC_OK)
            .then(|| log::info!("remove_template: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Removes all templates from flash storage.
pub fn remove_all_templates(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_remove_all(chain);
        (result == FPC_OK)
            .then(|| log::info!("remove_all_templates: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Wait for finger present on sensor and capture image. Set timeout to 0 to
/// wait indefinitely.
pub fn capture(chain: *mut HcpCom, timeout: u16) -> anyhow::Result<()> {
    unsafe {
        let result = bep_capture(chain, timeout);
        (result == FPC_OK)
            .then(|| log::info!("capture: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Enrolls a finger. Created template MUST be stored to flash storage or risk
/// being destroyed on power loss.
#[allow(non_snake_case)]
pub fn enroll_finger(chain: *mut HcpCom) -> anyhow::Result<()> {
    log::info!("Enrolling finger..");

    unsafe {
        loop {
            let result = bep_enroll_finger(chain);

            match result {
                FPC_OK => {
                    log::info!("enroll_finger: {}", debug(result));
                    return Ok(());
                }

                GENERAL_ERROR => {
                    log::warn!("GENERAL_ERROR, retrying...");
                }

                _ => {
                    log::error!("enroll_finger: {}", debug(result));
                    return Err(anyhow::anyhow!(debug(result)));
                }
            }
        }
    }
}

/// Identify prepared image against existing templates in flash storage.
pub fn identify(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_identify(chain);
        (result == FPC_OK)
            .then(|| log::info!("identify: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;
        Ok(())
    }
}

/// Capture and identify finger against existing templates in flash storage.
/// This function should always be prefered to its [`identify`] counterpart.
/// Returns true if the finger matches a template in flash storage.
/// Timeout in ms. Set timeout to 0 to wait indefinitely.
pub fn identify_finger(chain: *mut HcpCom, timeout: u32, template_id: u16) -> anyhow::Result<bool> {
    unsafe {
        let mut is_match = false;
        let mut id = template_id;

        let result = bep_identify_finger(chain, timeout, &mut id as *mut _, &mut is_match as *mut _);
        (result == FPC_OK)
            .then(|| log::info!("identify_finger: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(is_match)
    }
}

/// Pulls template stored in ram.
pub fn get_template(chain: *mut HcpCom) -> anyhow::Result<Vec<u8>> {
    unsafe {
        let mut data = vec![0u8; 128];

        let result = bep_template_get(chain, data.as_mut_ptr(), 0);
        (result == FPC_OK)
            .then(|| log::info!("get_template: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(data)
    }
}

/// Saves template from ram into flash.
pub fn save_template(chain: *mut HcpCom, template_id: u16) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_save(chain, template_id);
        (result == FPC_OK)
            .then(|| log::info!("save_template: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Copies template from flash storage to RAM.
pub fn load_storage(chain: *mut HcpCom, template_id: u16) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_load_storage(chain, template_id);
        (result == FPC_OK)
            .then(|| log::info!("load_storage: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Gets array of template IDs stored on the BM-Lite.
pub fn get_template_ids(chain: *mut HcpCom) -> anyhow::Result<Vec<u16>> {
    unsafe {
        let result = bep_template_get_ids(chain);
        (result == FPC_OK)
            .then(|| log::info!("load_storage: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        let array = (*chain).arg.data as *const u16;
        let array_size = (*chain).arg.size as usize / std::mem::size_of::<u32>();

        let mut template_ids = Vec::with_capacity(array_size);

        for i in 0..array_size {
            let template_id = (*array).add(1);
            template_ids.push(template_id);
        }

        log::info!("{:#?}", template_ids);

        Ok(template_ids)
    }
}

/// Gets the number of templates stored in flash storage.
pub fn template_count(chain: *mut HcpCom) -> anyhow::Result<u16> {
    unsafe {
        let mut count = 0;

        let result = bep_template_get_count(chain, &mut count as *mut _);
        (result == FPC_OK)
            .then(|| log::info!("template_count: {}", debug(result)))
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(count)
    }
}
