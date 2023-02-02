use info::{Info, InfoRaw};

use crate::php_lib;

pub(crate) mod info;

php_lib! {
    pub struct Standard<StandardRaw> {
        {
            pub info: Info<InfoRaw>,
        }
    }
}
