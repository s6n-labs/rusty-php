use std::ffi::CString;
use std::mem::MaybeUninit;

use rusty_php::callback::{Callback, SapiCallback};
use rusty_php::sapi::Sapi;
use rusty_php::sys::zend::Zval;
use rusty_php::zend::Value;
use rusty_php::PhpInit;

struct SapiCallbackImpl;

impl SapiCallback for SapiCallbackImpl {}

struct SapiImpl;

impl Sapi for SapiImpl {
    fn name(&self) -> &[u8] {
        b"rusty-php"
    }

    fn pretty_name(&self) -> &[u8] {
        b"rusty-php"
    }

    fn executable_location(&self) -> &[u8] {
        b"/opt/homebrew/bin"
    }

    fn callback(&self) -> Callback {
        Callback::new(SapiCallbackImpl)
    }
}

fn eval(contents: &str) -> Zval {
    let php = PhpInit::new(SapiImpl)
        .init()
        .unwrap()
        .startup_module()
        .unwrap()
        .startup_request()
        .unwrap();

    let mut retval = MaybeUninit::<Zval>::uninit();

    php.as_ref().zend.execute.zend_eval_string_ex(
        unsafe { CString::from_vec_unchecked(contents.as_bytes().to_vec()) }.into_raw(),
        retval.as_mut_ptr(),
        unsafe { CString::from_vec_unchecked(b"eval".to_vec()) }.into_raw(),
        true,
    );

    unsafe { retval.assume_init() }
}

#[test]
fn long() {
    let value = Value::from(eval("1234500000 + 67890"));
    assert_eq!(value, Value::Long(1234567890));
}

#[test]
fn double() {
    let value = Value::from(eval("1.234 + 5.678"));
    assert_eq!(value, Value::Double(6.912));
}

#[test]
fn string() {
    let value = Value::from(eval("'Hello, world!'"));
    assert_eq!(value, Value::String("Hello, world!".as_bytes()));
}
