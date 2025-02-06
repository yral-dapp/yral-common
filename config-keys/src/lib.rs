use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};

pub trait ConfigKey: Display {
    type Value: Serialize + DeserializeOwned;

    fn fallback() -> Option<Self::Value>;
}

#[macro_export]
macro_rules! key_derive {
    ($key:ident => $value:ty) => {
        impl crate::ConfigKey for $key {
            type Value = $value;

            fn fallback() -> Option<Self::Value> {
                None
            }
        }
    };

    ($key:ident => $value:ty|$fallback:expr) => {
        impl crate::ConfigKey for $key {
            type Value = $value;

            fn fallback() -> Option<Self::Value> {
                Some($fallback)
            }
        }
    };
}
