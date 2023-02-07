use std::ffi::CString;
use std::mem::MaybeUninit;

use rusty_php_sys::zend::Zval;

use crate::callback::{Callback, SapiCallback};
use crate::sapi::Sapi;
use crate::{PhpInit, PhpRequest};

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

pub struct TestBed {
    php: PhpRequest,
}

impl TestBed {
    pub fn startup() -> Self {
        Self {
            php: PhpInit::new(SapiImpl)
                .init()
                .unwrap()
                .startup_module()
                .unwrap()
                .startup_request()
                .unwrap(),
        }
    }

    pub fn shutdown(self) {
        self.php.shutdown_all();
    }

    pub fn eval(&self, contents: &str) -> Zval {
        let mut retval = MaybeUninit::<Zval>::uninit();

        self.php.as_ref().zend.execute.zend_eval_string_ex(
            unsafe { CString::from_vec_unchecked(contents.as_bytes().to_vec()) }.into_raw(),
            retval.as_mut_ptr(),
            unsafe { CString::from_vec_unchecked(b"TestBed".to_vec()) }.into_raw(),
            true,
        );

        unsafe { retval.assume_init() }
    }

    pub fn run_eval(contents: &str) -> Zval {
        let bed = Self::startup();
        let value = bed.eval(contents);

        bed.shutdown();
        value
    }
}
