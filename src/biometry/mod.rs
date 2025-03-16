use esp_idf_svc::sys::bmlite::{console_initparams_t, HCP_arg_t, HCP_comm_t};

mod commands;
mod ctx;
mod logic;

type HcpCom = HCP_comm_t;
type Params = console_initparams_t;
type HcpArg = HCP_arg_t;

pub use logic::{check_user, enroll_user, init, reset};

#[allow(unused)]
mod functions {
    pub use esp_idf_svc::sys::bmlite::{
        bep_capture,
        bep_enroll_finger,
        bep_identify,
        bep_identify_finger,
        bep_image_extract,
        bep_image_get,
        bep_image_get_size,
        bep_image_put,
        bep_sensor_calibrate,
        bep_sensor_calibrate_remove,
        bep_sensor_reset,
        bep_sw_reset,
        bep_template_get,
        bep_template_get_count,
        bep_template_get_ids,
        bep_template_load_storage,
        bep_template_put,
        bep_template_remove,
        bep_template_remove_all,
        bep_template_remove_ram,
        bep_template_save,
        bep_uart_speed_get,
        bep_uart_speed_set,
        bep_unique_id_get,
        bep_version,
        platform_init,
        platform_deinit,
    };
}


mod results {
    pub use esp_idf_svc::sys::bmlite::{
        fpc_bep_result_t_FPC_BEP_FINGER_NOT_STABLE as FINGER_NOT_STABLE,
        fpc_bep_result_t_FPC_BEP_RESULT_BROKEN_SENSOR as BROKEN_SENSOR,
        fpc_bep_result_t_FPC_BEP_RESULT_CANCELLED as CANCELLED,
        fpc_bep_result_t_FPC_BEP_RESULT_CRYPTO_ERROR as CRYPTO_ERROR,
        fpc_bep_result_t_FPC_BEP_RESULT_GENERAL_ERROR as GENERAL_ERROR,
        fpc_bep_result_t_FPC_BEP_RESULT_ID_NOT_FOUND as ID_NOT_FOUND,
        fpc_bep_result_t_FPC_BEP_RESULT_ID_NOT_UNIQUE as ID_NOT_UNIQUE,
        fpc_bep_result_t_FPC_BEP_RESULT_IMAGE_CAPTURE_ERROR as IMAGE_CAPTURE_ERROR,
        fpc_bep_result_t_FPC_BEP_RESULT_INTERNAL_ERROR as INTERNAL_ERROR,
        fpc_bep_result_t_FPC_BEP_RESULT_INVALID_ARGUMENT as INVALID_ARGUMENT,
        fpc_bep_result_t_FPC_BEP_RESULT_INVALID_CALIBRATION as INVALID_CALIBRATION,
        fpc_bep_result_t_FPC_BEP_RESULT_INVALID_FORMAT as INVALID_FORMAT,
        fpc_bep_result_t_FPC_BEP_RESULT_INVALID_PARAMETER as INVALID_PARAMETER,
        fpc_bep_result_t_FPC_BEP_RESULT_IO_ERROR as IO_ERROR,
        fpc_bep_result_t_FPC_BEP_RESULT_MISSING_TEMPLATE as MISSING_TEMPLATE,
        fpc_bep_result_t_FPC_BEP_RESULT_NOT_IMPLEMENTED as NOT_IMPLEMENTED,
        fpc_bep_result_t_FPC_BEP_RESULT_NOT_INITIALIZED as NOT_INITIALIZED,
        fpc_bep_result_t_FPC_BEP_RESULT_NOT_SUPPORTED as NOT_SUPPORTED,
        fpc_bep_result_t_FPC_BEP_RESULT_NO_MEMORY as NO_MEMORY,
        fpc_bep_result_t_FPC_BEP_RESULT_NO_RESOURCE as NO_RESOURCE,
        fpc_bep_result_t_FPC_BEP_RESULT_OK as FPC_OK,
        fpc_bep_result_t_FPC_BEP_RESULT_SENSOR_MISMATCH as SENSOR_MISMATCH,
        fpc_bep_result_t_FPC_BEP_RESULT_SENSOR_NOT_INITIALIZED as SENSOR_NOT_INITIALIZED,
        fpc_bep_result_t_FPC_BEP_RESULT_STORAGE_NOT_FORMATTED as STORAGE_NOT_FORMATTED,
        fpc_bep_result_t_FPC_BEP_RESULT_TIMEOUT as TIMEOUT,
        fpc_bep_result_t_FPC_BEP_RESULT_TOO_MANY_BAD_IMAGES as TOO_MANY_BAD_IMAGES,
        fpc_bep_result_t_FPC_BEP_RESULT_WRONG_STATE as WRONG_STATE,
    };

    #[allow(non_snake_case)]
    pub fn debug(res: i32) -> String {
        match res {
            FPC_OK => "OK.".to_string(),
            GENERAL_ERROR => "General error.".to_string(),
            INTERNAL_ERROR => "Internal error.".to_string(),
            INVALID_ARGUMENT => "Invalid argument.".to_string(),
            NOT_IMPLEMENTED => "Functionality is not implemented.".to_string(),
            CANCELLED => "Operation was cancelled.".to_string(),
            NO_MEMORY => "Out of memory.".to_string(),
            NO_RESOURCE => "Resources are not available.".to_string(),
            IO_ERROR => "An I/O error occurred.".to_string(),
            BROKEN_SENSOR => "Sensor is broken.".to_string(),
            WRONG_STATE => "Operation cannot be performed in current state.".to_string(),
            TIMEOUT => "Operation timed out.".to_string(),
            ID_NOT_UNIQUE => "ID is not unique.".to_string(),
            ID_NOT_FOUND => "ID is not found.".to_string(),
            INVALID_FORMAT => "Invalid format.".to_string(),
            IMAGE_CAPTURE_ERROR => "Image capture error occurred.".to_string(),
            SENSOR_MISMATCH => "Sensor hardware ID or sensor configuration mismatch.".to_string(),
            INVALID_PARAMETER => "Invalid parameter.".to_string(),
            MISSING_TEMPLATE => "Missing Template.".to_string(),
            INVALID_CALIBRATION => "Invalid Calibration.".to_string(),
            STORAGE_NOT_FORMATTED => "Calibration/template storage not formatted.".to_string(),
            SENSOR_NOT_INITIALIZED => "Sensor not initialized.".to_string(),
            TOO_MANY_BAD_IMAGES => "Enroll failed after too many bad images.".to_string(),
            CRYPTO_ERROR => "Cryptographic operation failed.".to_string(),
            NOT_SUPPORTED => "Functionality is not supported.".to_string(),
            FINGER_NOT_STABLE => "Finger not stable during image capture.".to_string(),
            NOT_INITIALIZED => "Functionality cannot be used before it's initialized.".to_string(),
            _ => format!("Unknown result code: {}", res),
        }
    }
}
