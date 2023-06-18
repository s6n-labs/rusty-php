use crate::zend::ZendString;

pub const ZEND_STR_AUTOGLOBAL_SERVER: usize = 66;

extern "C" {
    pub static zend_known_strings: *mut *mut ZendString;
}

#[macro_export]
macro_rules! zstr_known {
    ($idx: expr) => {
        ::std::slice::from_raw_parts($crate::zend::string::zend_known_strings, $idx + 1)[$idx]
    };
}

pub use zstr_known;
