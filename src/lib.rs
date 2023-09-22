//! This crate implements various types that can be used in a distributed environment.
//! As a backend, it uses different backends. They are implemented in the submodules and can be installed separately.
//! The usage of the types can vary depending on the backend. So examples are provided in the submodules.
//!
//! # Crate features
//!
//! By default, the crate uses the Redis backend. If you want to use another backend, you can enable other backends. For best performance, you should only enable one backend.
//!
//! Backend features:
//! * [redis]: Enables the Redis backend. (Default)

/// This module contains the types that can be used with a Redis backend. Must be enabled by feature `redis`.
#[cfg(feature = "redis")]
pub mod redis;
