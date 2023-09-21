//! # Redis Types
//!
//! This crate provides a set of types that can be stored in Redis. The types are:
//!
//! * [bool](crate::bool)
//! * Integer types:
//!     * signed Integer: [i8](crate::i8), [i16](crate::i16), [i32](crate::i32), [i64](crate::i64), [isize](crate::isize)
//!     * unsigned Integer: [u8](crate::u8), [u16](crate::u16), [u32](crate::u32), [u64](crate::u64), [usize](crate::usize)
//! * [String](crate::String)
//!
//! This crate implements the most common traits for the primitive types, so it is frictionless to use them in place.
//! With this crate it is possible to create multiple services that shares the values via Redis.
//! This is helpful if you want to create a distributed system and run multiple instances of the same service.
//! Or you want to communicate between different services. All this kind of stuff can be done with this crate.
//!
//! # Upcoming Features
//!
//! In a later release it will be possible to lock values like a Mutex or RwLock.
//! Also it will be possible to create happens-before relationships between store and load operations like atomic types.
//! So it will be possible to use the types in a concurrent environment in the same way as in a distributed one.
//!
//! Also it will be possible to create other backends than Redis.
//!
//! # Usage
//!
//! ```
//! use dtypes::Di32 as i32;
//!
//! let client = redis::Client::open("redis://localhost:6379").unwrap();
//! let mut i32 = i32::with_value(1, "test_add", client.clone());
//!
//! i32 = i32 + i32::with_value(2, "test_add2", client.clone());
//! assert_eq!(i32, 3);
//! ```
//!
//! # Custom Types
//!
//! It is possible to implement your own complex types by implementing the [BackedType](crate::BackedType) trait.
//! But it should not be needed as long as your type implements some or all of the various [Ops](https://doc.rust-lang.org/std/ops/index.html) traits.
mod bool_type;
mod generic;
mod helper;
mod integer;
mod lock;
mod string;

pub(crate) use helper::apply_operator;

pub use bool_type::TBool as Dbool;
pub use generic::RedisGeneric;
pub use integer::{
    Ti16 as Di16, Ti32 as Di32, Ti64 as Di64, Ti8 as Di8, Tisize as Disize, Tu16 as Du16,
    Tu32 as Du32, Tu64 as Du64, Tu8 as Du8, Tusize as Dusize,
};
pub use string::TString as DString;
