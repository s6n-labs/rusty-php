use crate::zend::{ZendResult, ZendString};

extern "C" {
    pub fn zend_is_auto_global(name: *mut ZendString) -> ZendResult;
}
