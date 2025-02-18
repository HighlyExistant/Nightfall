use std::{alloc::Layout, any::type_name, cell::UnsafeCell, marker::PhantomData, ops::Div, ptr::NonNull};

use anyhow::Ok;

use crate::{arena::Arena, error::AllocError};

use super::PoolAllocator;

pub struct Pool<T> {
    ptr: *mut u8,
    capacity: usize,
    free: UnsafeCell<Vec<usize>>,
    marker_: PhantomData<T>,
}

impl<T> Pool<T> {
    pub unsafe fn from_raw(ptr: *mut u8, size: usize) -> Self {
        assert!(size % std::mem::size_of::<T>() == 0, "size must be aligned to {}", type_name::<T>());
        Self { ptr, capacity: size.div(std::mem::size_of::<T>()), free: UnsafeCell::new(vec![0]), marker_: PhantomData }
    }
    pub unsafe fn from_slice(slice: &mut [T]) -> Self {
        Self { ptr: slice.as_mut_ptr().cast(), capacity: slice.len(), free: UnsafeCell::new(vec![0]), marker_: PhantomData }
    }
    pub fn from_arena(arena: &dyn Arena<Allocation = *mut u8>, layout: Layout) -> anyhow::Result<Self> {
        let ptr = arena.arena_alloc(layout)?;
        Ok(Self { ptr, capacity: layout.size(), free: UnsafeCell::new(vec![0]), marker_: PhantomData })
    }
    pub fn free(&self) -> &mut Vec<usize> {
        unsafe { self.free.get().as_mut().unwrap() }
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl<T> PoolAllocator for Pool<T> {
    type Allocation = NonNull<T>;
    fn allocate(&self) -> anyhow::Result<Self::Allocation> {
        if self.free().len() == 0 {
            Err(AllocError::OutOfMemory)?
        }
        if let Some(alloc) = self.free().pop() {
            if alloc >= self.capacity {
                Err(AllocError::OutOfMemory)?
            }
            self.free().push(alloc+1);
            unsafe {
                Ok(NonNull::new(self.as_ptr().cast::<T>().add(alloc)).unwrap())
            }
        } else {
            Err(AllocError::OutOfMemory)?
        }
    }
    fn deallocate(&self, allocation: Self::Allocation) -> anyhow::Result<()> {
        let offset = (allocation.as_ptr() as usize - self.as_ptr() as usize).div(std::mem::size_of::<T>());
        self.free().push(offset);
        Ok(())
    }
}