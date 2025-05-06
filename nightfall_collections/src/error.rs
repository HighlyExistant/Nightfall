use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectionError {
    #[error("Not enough capacity to insert elements")]
    CapacityFull
}