use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllocError {
    #[error("Out of Memory")]
    OutOfMemory
}