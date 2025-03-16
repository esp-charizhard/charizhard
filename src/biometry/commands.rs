use super::functions::*;
use super::results::{debug, FPC_OK, GENERAL_ERROR};
use super::{HcpCom, Params};

/// Initializes the sensor. This function should be called once and only once
/// per program, at the beginning of the mainloop.
pub fn init_sensor(params: *mut Params) -> anyhow::Result<()> {
    unsafe {
        let result = platform_init(params as *mut _);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Deinitializes the sensor. Calling any other function other than init_sensor after this will invariably result in UB.
pub fn deinit_sensor(params: *mut Params) -> anyhow::Result<()> {
    unsafe {
        let result = platform_deinit(params as *mut _);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Calibrates the sensor. A reboot is required for this to take effect.
pub fn calibrate_sensor(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sensor_calibrate(chain);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Resets sensor calibration. A reboot is required for this to take effect.
pub fn reset_sensor_calibration(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_sensor_calibrate_remove(chain);
        (result == FPC_OK)
            .then_some(())
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
            .then_some(())
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
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Removes all templates from flash storage.
pub fn remove_all_templates(chain: *mut HcpCom) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_remove_all(chain);
        (result == FPC_OK)
            .then_some(())
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
                    log::info!("Enroll successful.");
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

/// Capture and identify finger against existing templates in flash storage.
/// Returns true if the finger matches a template in flash storage.
/// Timeout in ms. Set timeout to 0 to wait indefinitely.
pub fn identify_finger(chain: *mut HcpCom, timeout: u32, template_id: u16) -> anyhow::Result<bool> {
    unsafe {
        let mut is_match = false;
        let mut id = template_id;

        let result = bep_identify_finger(chain, timeout, &mut id as *mut _, &mut is_match as *mut _);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(is_match)
    }
}

/// Saves template from ram into flash.
pub fn save_template(chain: *mut HcpCom, template_id: u16) -> anyhow::Result<()> {
    unsafe {
        let result = bep_template_save(chain, template_id);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(())
    }
}

/// Gets the number of templates stored in flash storage.
pub fn template_count(chain: *mut HcpCom) -> anyhow::Result<u16> {
    unsafe {
        let mut count = 0;

        let result = bep_template_get_count(chain, &mut count as *mut _);
        (result == FPC_OK)
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!(debug(result)))?;

        Ok(count)
    }
}
