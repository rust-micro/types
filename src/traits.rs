//! # Traits
//!
//! This module contains the traits used by the library.

/// The BackedType trait is used to define the methods that are common to all types.
pub trait BackedType<T> {
    /// The get_conn method returns a mutable reference to the redis::Connection.
    fn get_conn(&mut self) -> &mut redis::Connection;
    /// The get method returns a reference to the value stored in the type.
    fn get(&self) -> &T;
}
