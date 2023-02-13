#![feature(c_variadic)]
#![feature(try_trait_v2)]

mod result;

pub mod callback;
pub mod ffi;
pub mod sapi;
pub mod test;
pub mod zend;

use std::error::Error;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::result::Result as StdResult;
use std::sync::Arc;

use pathsearch::find_executable_in_path;
pub use rusty_php_sys as sys;

pub use crate::result::{Err, Ok, Result};
use crate::sapi::{Sapi, SapiExt};
use crate::sys::sapi::SapiModuleStruct;
use crate::sys::zend::stream::ZendFileHandle;
use crate::sys::Php as PhpInner;
use crate::zend::file::ZFileHandle;

pub struct PhpRequest<'r> {
    inner: &'r PhpModule,
}

impl<'r> PhpRequest<'r> {
    fn startup(inner: &'r PhpModule) -> Result<Self> {
        Result::from(inner.as_ref().php_request_startup())?;

        Ok(Self { inner })
    }

    pub fn execute_script<'a>(&self, script: &'a mut ZFileHandle<'a>) {
        self.as_ref()
            .php_execute_script(<&mut ZendFileHandle>::from(script) as *mut ZendFileHandle);
    }
}

impl<'r> Drop for PhpRequest<'r> {
    fn drop(&mut self) {
        self.inner.as_ref().php_request_shutdown(null_mut());
    }
}

impl<'r> AsRef<PhpInner> for PhpRequest<'r> {
    #[inline]
    fn as_ref(&self) -> &PhpInner {
        self.inner.as_ref()
    }
}

pub struct PhpModule {
    inner: Php,
}

impl PhpModule {
    fn startup(inner: Php) -> Result<Self> {
        Result::<()>::from(inner.as_ref().php_module_startup(
            Arc::into_raw(Arc::clone(&inner.sapi_module)) as *mut SapiModuleStruct,
            null_mut(),
        ))?;

        Ok(Self { inner })
    }

    #[must_use]
    pub fn startup_request(&self) -> Result<PhpRequest> {
        PhpRequest::startup(self)
    }

    pub fn init_stream_path<P>(&self, path: P) -> ZFileHandle
    where
        P: AsRef<Path>,
    {
        let mut file_handle = MaybeUninit::<ZendFileHandle>::uninit();

        self.as_ref().zend.stream.zend_stream_init_filename(
            file_handle.as_mut_ptr(),
            unsafe {
                CString::from_vec_unchecked(path.as_ref().to_string_lossy().as_bytes().to_vec())
            }
            .into_raw(),
        );

        unsafe { file_handle.assume_init() }.into()
    }
}

impl Drop for PhpModule {
    fn drop(&mut self) {
        self.inner.as_ref().php_module_shutdown();
    }
}

impl AsRef<PhpInner> for PhpModule {
    #[inline]
    fn as_ref(&self) -> &PhpInner {
        self.inner.as_ref()
    }
}

pub struct Php {
    inner: PhpInner,
    sapi_module: Arc<SapiModuleStruct>,
}

impl Php {
    fn startup<P, S>(path: P, sapi: S) -> StdResult<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
        S: SapiExt,
    {
        sapi.register();

        let sapi_module = Arc::new(sapi.into_raw());
        let php = PhpInner::load(path)?;

        php.sapi_startup(Arc::into_raw(Arc::clone(&sapi_module)) as *mut SapiModuleStruct);

        StdResult::Ok(Self {
            inner: php,
            sapi_module,
        })
    }

    #[must_use]
    pub fn startup_module(self) -> Result<PhpModule> {
        PhpModule::startup(self)
    }
}

impl Drop for Php {
    fn drop(&mut self) {
        self.as_ref().sapi_shutdown();
    }
}

impl AsRef<PhpInner> for Php {
    #[inline]
    fn as_ref(&self) -> &PhpInner {
        &self.inner
    }
}

pub struct PhpInit<S>
where
    S: Sapi,
{
    sapi: S,
    path: Option<PathBuf>,
}

impl<S> PhpInit<S>
where
    S: Sapi,
{
    pub fn new(sapi: S) -> Self {
        Self { sapi, path: None }
    }

    pub fn with_path<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn init(self) -> StdResult<Php, Box<dyn Error>> {
        Php::startup(
            match self.path {
                Some(p) => p,
                None => find_executable_in_path("php").expect("PHP does not exist"),
            },
            self.sapi,
        )
    }
}
