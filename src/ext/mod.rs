use standard::{Standard, StandardRaw};

use crate::php_lib;

pub(crate) mod standard;

php_lib! {
    pub struct Extensions<ExtensionsRaw> {
        {
            pub standard: Standard<StandardRaw>,
        }
    }
}
