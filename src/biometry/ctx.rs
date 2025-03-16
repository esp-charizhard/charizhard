use core::ptr;
use std::sync::{Arc, Mutex};

use super::{HcpCom, Params};
use crate::biometry::commands::deinit_sensor;

pub struct SensorCtx {
    pub params: *mut Params,
    pub chain: *mut HcpCom,
}

unsafe impl Send for SensorCtx {}
unsafe impl Sync for SensorCtx {}

impl SensorCtx {
    /// This function should never be called. It only serves to initialize the
    /// [`lazy_static::lazy_static!`] macro.
    fn new() -> Self {
        SensorCtx {
            params: ptr::null_mut(),
            chain: ptr::null_mut(),
        }
    }

    pub fn set(&mut self, params: *mut Params, chain: *mut HcpCom) {
        if self.is_set() {
            log::error!("BM-Lite pointers were already initialized!");
            return;
        }

        log::warn!("Storing BM-Lite pointers!");

        self.params = params;
        self.chain = chain;
    }

    pub fn is_set(&self) -> bool {
        !(self.params.is_null() || self.chain.is_null())
    }

    pub fn reset(&mut self) {
        if !self.is_set() {
            log::error!("BM-Lite pointers weren't initialized!");
            return;
        }

        unsafe {
            log::warn!("Resetting BM-Lite pointers!");

            _ = deinit_sensor(self.params);

            _ = Box::from_raw(self.params);
            _ = Box::from_raw(self.chain);

            self.params = ptr::null_mut();
            self.chain = ptr::null_mut();
        }
    }
}

lazy_static::lazy_static!(
    /// This is the global hot potato that needs to never ever be dropped.
    /// Care should be taken when accessing this variable as thread safety is not guaranteed.
    pub static ref SENSOR_CTX: Arc<Mutex<SensorCtx>> = Arc::new(Mutex::new(SensorCtx::new()));
);
