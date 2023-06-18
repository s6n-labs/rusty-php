#![feature(c_variadic)]
#![feature(try_trait_v2)]

mod result;

pub mod callback;
pub mod ffi;
pub mod sapi;
pub mod test;
pub mod zend;

use std::error::Error;
use std::ptr::null_mut;
use std::result::Result as StdResult;
use std::sync::Arc;

pub use rusty_php_sys as sys;

pub use crate::result::{Err, Ok, Result};
use crate::sapi::{Sapi, SapiExt};
use crate::sys::sapi::SapiModuleStruct;

pub struct PhpRequest {
    inner: PhpModule,
}

impl PhpRequest {
    fn startup(inner: PhpModule) -> Result<Self> {
        Result::from(unsafe { sys::php_request_startup() })?;

        Ok(Self { inner })
    }

    #[must_use]
    pub fn shutdown(self) -> PhpModule {
        unsafe {
            sys::php_request_shutdown(null_mut());
        }
        self.inner
    }

    pub fn shutdown_all(self) {
        self.shutdown().shutdown_all()
    }
}

pub struct PhpModule {
    inner: Php,
}

impl PhpModule {
    fn startup(inner: Php) -> Result<Self> {
        Result::<()>::from(unsafe {
            sys::php_module_startup(
                Arc::into_raw(Arc::clone(&inner.sapi_module)) as *mut SapiModuleStruct,
                null_mut(),
            )
        })?;

        Ok(Self { inner })
    }

    #[must_use]
    pub fn startup_request(self) -> Result<PhpRequest> {
        PhpRequest::startup(self)
    }

    #[must_use]
    pub fn shutdown(self) -> Php {
        unsafe { sys::php_module_shutdown() };
        self.inner
    }

    pub fn shutdown_all(self) {
        self.shutdown().shutdown()
    }
}

pub struct Php {
    sapi_module: Arc<SapiModuleStruct>,
}

impl Php {
    fn startup<S>(sapi: S) -> StdResult<Self, Box<dyn Error>>
    where
        S: SapiExt,
    {
        #[cfg(feature = "zts")]
        unsafe {
            sys::php_tsrm_startup()
        };

        sapi.register();

        let sapi_module = Arc::new(sapi.into_raw());
        unsafe {
            sys::sapi_startup(Arc::into_raw(Arc::clone(&sapi_module)) as *mut SapiModuleStruct)
        };

        StdResult::Ok(Self { sapi_module })
    }

    #[must_use]
    pub fn startup_module(self) -> Result<PhpModule> {
        PhpModule::startup(self)
    }

    pub fn shutdown(self) {
        unsafe { sys::sapi_shutdown() };
    }
}

pub struct PhpInit<S>
where
    S: Sapi,
{
    sapi: S,
}

impl<S> PhpInit<S>
where
    S: Sapi,
{
    pub fn new(sapi: S) -> Self {
        Self { sapi }
    }

    pub fn init(self) -> StdResult<Php, Box<dyn Error>> {
        Php::startup(self.sapi)
    }
}
