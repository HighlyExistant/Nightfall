use std::ops::{Deref, DerefMut};

use crossbeam::queue::SegQueue;

pub struct PoolAlloc<T>(T);
impl<T> Deref for PoolAlloc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for PoolAlloc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> Drop for PoolAlloc<T> {
    fn drop(&mut self) {
        
    }
}
pub struct StandardPool<T> {
    available: SegQueue<T>,
    free: SegQueue<T>,
}

