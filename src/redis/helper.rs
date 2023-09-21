use crate::redis::Generic;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;

pub(crate) fn apply_operator<T>(mut me: Generic<T>, rhs: T, func: impl Fn(T, T) -> T) -> Generic<T>
where
    T: Display + Serialize + DeserializeOwned,
{
    let value = me.cache.take();

    let value = match value {
        Some(value) => func(value, rhs),
        None => rhs,
    };

    me.store(value);
    me
}
