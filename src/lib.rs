mod integer;
mod string;
mod traits;

pub use integer::Ti32 as i32;
pub use string::TString as String;

macro_rules! impl_backed_type {
    ($($outer:ty)*) => ($(
        impl traits::BackedType for $outer {
            fn get_conn(&mut self) -> &mut redis::Connection {
                if self.conn.is_none() {
                    self.conn = Some(self.client.get_connection().unwrap());
                }
                self.conn.as_mut().unwrap()
            }
        }
    )*)
}

impl_backed_type!(i32 String);
