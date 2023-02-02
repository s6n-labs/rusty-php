use std::ffi::c_int;
use std::io::{stdout, Write};
use std::process::exit;
use std::sync::Arc;

use libc::{gid_t, uid_t};
use tracing::{debug, error, warn};

use crate::result::{Ok, Result};
use crate::sys::sapi::{
    SapiHeaderOpEnum, SapiHeaderStruct, SapiHeadersStruct, SAPI_HEADER_SENT_SUCCESSFULLY,
};
use crate::sys::zend::{HashTable, ZendStat, Zval};

pub(crate) mod listeners;

macro_rules! no_op {
    () => {
        tracing::warn!("NOOP: No action declared about this callback.");
    };
}

macro_rules! default_behaviour {
    () => {
        tracing::warn!(
            "NOOP: No action declared about this callback, and the default behaviour was used."
        );
    };
}

#[allow(unused_variables)]
pub trait SapiCallback {
    fn on_startup(&self) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_shutdown(&self) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_activate(&self) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_deactivate(&self) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_ub_write(&self, str: &[u8]) -> usize {
        default_behaviour!();

        let bytes = stdout().write(str).unwrap();
        match bytes == str.len() {
            true => debug!("WROTE: {} bytes", bytes),
            _ => warn!("WROTE: {} bytes (!= {} bytes)", bytes, str.len()),
        }

        bytes
    }

    fn on_flush(&self) {
        default_behaviour!();
        stdout().flush().unwrap();
    }

    fn on_get_stat(&self) -> Result<ZendStat> {
        no_op!();
        todo!()
    }

    fn on_get_env(&self, name: &[u8]) -> Option<Vec<u8>> {
        no_op!();
        None
    }

    fn on_sapi_error(&self, ty: i32, error_msg: &[u8]) {
        // TODO: Variadic arguments
        default_behaviour!();
        error!("ERROR: [{}] {}", ty, String::from_utf8_lossy(error_msg))
    }

    fn on_header_handler(
        &self,
        header: &SapiHeaderStruct,
        op: SapiHeaderOpEnum,
        headers: &mut SapiHeadersStruct,
    ) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_send_headers(&self, headers: &SapiHeadersStruct) -> c_int {
        // TODO: Enum type
        no_op!();
        SAPI_HEADER_SENT_SUCCESSFULLY
    }

    fn on_send_header(&self, header: &SapiHeaderStruct) {
        no_op!();
    }

    fn on_read_post(&self, buffer: &mut [u8]) -> usize {
        no_op!();
        0
    }

    fn on_read_cookies(&self) -> Option<Vec<u8>> {
        no_op!();
        None
    }

    fn on_register_server_variables(&self, track_vars_array: &mut Zval) {
        no_op!();
    }

    fn on_log_message(&self, message: &[u8], syslog_type_int: i32) {
        default_behaviour!();
        debug!(
            "LOG: [{}] {}",
            syslog_type_int,
            String::from_utf8_lossy(message),
        )
    }

    fn on_get_request_time(&self) -> Result<f64> {
        no_op!();
        Ok(0_f64)
    }

    fn on_terminate_process(&self) {
        default_behaviour!();
        exit(1);
    }

    fn on_default_post_reader(&self) {
        no_op!();
    }

    fn on_treat_data(&self, arg: i32, str: &[u8], dest_array: *mut Zval) {
        no_op!();
    }

    fn on_get_fd(&self) -> Result<i32> {
        no_op!();
        Ok(0)
    }

    fn on_force_http_10(&self) -> Result<()> {
        no_op!();
        Ok(())
    }

    fn on_get_target_uid(&self) -> Result<uid_t> {
        no_op!();
        Ok(0)
    }

    fn on_get_target_gid(&self) -> Result<gid_t> {
        no_op!();
        Ok(0)
    }

    fn on_input_filter(&self, arg: i32, var: &[u8], val: &mut [&mut [u8]]) -> Result<usize> {
        no_op!();
        Ok(val.len())
    }

    fn on_ini_defaults(&self, configuration_hash: &mut HashTable) {
        no_op!();
    }

    fn on_input_filter_init(&self) -> Result<()> {
        no_op!();
        Ok(())
    }
}

pub struct Callback {
    listener: Arc<dyn SapiCallback>,
}

impl Callback {
    pub fn new<S>(listener: S) -> Self
    where
        S: SapiCallback + 'static,
    {
        Self {
            listener: Arc::new(listener),
        }
    }
}

static mut GLOBAL_CALLBACK: Option<Callback> = None;

pub(crate) fn register_global_callback(callback: Callback) {
    unsafe {
        GLOBAL_CALLBACK = Some(callback);
    }
}
