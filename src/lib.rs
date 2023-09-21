pub mod redis;

pub enum LockEnum<T> {
    Redis(redis::Generic<T>),
}
