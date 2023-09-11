pub trait BackedType {
    fn get_conn(&mut self) -> &mut redis::Connection;
}
