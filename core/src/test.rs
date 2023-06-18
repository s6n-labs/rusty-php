use std::ffi::CString;
use std::mem::MaybeUninit;

use rusty_php_sys::zend::execute::zend_eval_string_ex;
use rusty_php_sys::zend::Zval;

use crate::callback::{Callback, SapiCallback};
use crate::sapi::Sapi;
use crate::{PhpInit, PhpModule, PhpRequest};

struct SapiCallbackImpl;

impl SapiCallback for SapiCallbackImpl {}

struct SapiImpl;

impl Sapi for SapiImpl {
    fn name(&self) -> &[u8] {
        b"rusty-php-testbed"
    }

    fn pretty_name(&self) -> &[u8] {
        b"TestBed for rusty-php"
    }

    fn executable_location(&self) -> &[u8] {
        b"/opt/homebrew/bin"
    }

    fn callback(&self) -> Callback {
        Callback::new(SapiCallbackImpl)
    }
}

pub struct TestBedRequest {
    _request: PhpRequest,
}

impl TestBedRequest {
    pub fn startup(bed: TestBed) -> Self {
        let request = bed.php.startup_request().unwrap();
        Self { _request: request }
    }

    pub fn eval(&self, contents: &str) -> Zval {
        let mut retval = MaybeUninit::<Zval>::uninit();

        unsafe {
            zend_eval_string_ex(
                CString::from_vec_unchecked(contents.as_bytes().to_vec()).into_raw(),
                retval.as_mut_ptr(),
                CString::from_vec_unchecked(b"TestBed".to_vec()).into_raw(),
                true,
            );
        }

        unsafe { retval.assume_init() }
    }
}

pub struct TestBed {
    php: PhpModule,
}

impl TestBed {
    pub fn init() -> Self {
        Self {
            php: PhpInit::new(SapiImpl)
                .init()
                .unwrap()
                .startup_module()
                .unwrap(),
        }
    }

    pub fn startup(self) -> TestBedRequest {
        TestBedRequest::startup(self)
    }

    pub fn run<F, R>(f: F) -> R
    where
        F: FnOnce(&TestBedRequest) -> R,
    {
        let bed = Self::init();
        let request = bed.startup();
        f(&request)
    }
}
