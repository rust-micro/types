use crate::traits::BackedType;
use std::fmt::Debug;

pub struct RedisGeneric<T> {
    pub(crate) value: T,
    pub(crate) field_name: String,
    conn: Option<redis::Connection>,
    client: redis::Client,
}

impl<T> RedisGeneric<T> {
    pub fn new(value: T, client: redis::Client, field_name: String) -> RedisGeneric<T> {
        RedisGeneric {
            value,
            client,
            conn: None,
            field_name,
        }
    }
}

impl<T> BackedType<T> for RedisGeneric<T> {
    fn get_conn(&mut self) -> &mut redis::Connection {
        if self.conn.is_none() {
            let conn = self
                .client
                .get_connection()
                .expect("Failed to get connection");
            self.conn = Some(conn);
        }
        self.conn.as_mut().unwrap()
    }

    fn get(&self) -> &T {
        &self.value
    }
}

impl<T: PartialEq> PartialEq<T> for RedisGeneric<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: Debug> Debug for RedisGeneric<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generic")
            .field("value", &self.value)
            .field("field_name", &self.field_name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partialeq() {
        let s1 = RedisGeneric::new(
            2,
            redis::Client::open("redis://localhost/").unwrap(),
            "s1".to_string(),
        );
        assert_eq!(s1, 2);
    }
}
