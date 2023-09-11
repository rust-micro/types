mod bool_type;
mod generic;
mod integer;
mod string;
mod traits;

pub use bool_type::TBool as bool;
pub(crate) use generic::RedisGeneric;
pub use integer::Ti32 as i32;
pub use string::TString as String;
pub use traits::BackedType;
