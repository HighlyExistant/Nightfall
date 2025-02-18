use std::{rc::Rc, sync::Arc};
mod ptr;
mod standard;

pub use ptr::*;
pub use standard::*;
pub trait PoolAllocator {
    type Allocation;
    fn allocate(&self) -> anyhow::Result<Self::Allocation>;
    fn deallocate(&self, allocation: Self::Allocation) -> anyhow::Result<()>;
}
pub trait PoolAllocatorGuarded: PoolAllocator {
    type Guard;
    fn allocate_guarded(&self) -> anyhow::Result<Self::Guard>;
}

impl<T: PoolAllocator> PoolAllocator for Arc<T> {
    type Allocation = T::Allocation;
    fn allocate(&self) -> anyhow::Result<Self::Allocation> {
        (**self).allocate()
    }
    fn deallocate(&self, allocation: Self::Allocation) -> anyhow::Result<()> {
        (**self).deallocate(allocation)
    }
}

impl<T: PoolAllocator> PoolAllocator for Rc<T> {
    type Allocation = T::Allocation;
    fn allocate(&self) -> anyhow::Result<Self::Allocation> {
        (**self).allocate()
    }
    fn deallocate(&self, allocation: Self::Allocation) -> anyhow::Result<()> {
        (**self).deallocate(allocation)
    }
}
impl<T: PoolAllocator> PoolAllocator for Box<T> {
    type Allocation = T::Allocation;
    fn allocate(&self) -> anyhow::Result<Self::Allocation> {
        (**self).allocate()
    }
    fn deallocate(&self, allocation: Self::Allocation) -> anyhow::Result<()> {
        (**self).deallocate(allocation)
    }
}