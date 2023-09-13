use crate::RedisGeneric;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;

pub(crate) fn apply_operator<T>(
    mut me: RedisGeneric<T>,
    rhs: T,
    func: impl Fn(T, T) -> T,
) -> RedisGeneric<T>
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
