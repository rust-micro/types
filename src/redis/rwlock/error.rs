use thiserror::Error;
#[derive(Error, Debug)]
pub enum RwLockError {
    #[error("The lock is already locked for writer.")]
    WriterAlreadyLocked,
    #[error("The lock is already locked for reader.")]
    StillReader,
    #[error("The lock could not be dropped.")]
    LockNotDroppable,
}
