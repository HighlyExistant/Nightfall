use std::{alloc::Layout, ptr::NonNull};

use anyhow::Ok;

use crate::error::CollectionError;
/// Fixed size circular buffer 
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    buf: NonNull<T>,
    capacity: usize,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(count: usize) -> Self {
        // assert!(count > 1, "A circular buffer should not have a fixed size of 1, consider using just a regular buffer");
        let buf = unsafe {
            let data =  std::alloc::alloc_zeroed(Layout::array::<T>(count).unwrap()) as *mut T;
            NonNull::new(data).unwrap()
        };
        Self { buf, capacity: count, head: 0, tail: 0, len: 0 }
    }
    #[inline(always)]
    pub fn full(&self) -> bool {
        self.len == self.capacity()
    }
    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.len != self.capacity()
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: T) -> anyhow::Result<()> {
        let capacity = self.capacity();
        if self.len == capacity {
            Err(CollectionError::CapacityFull)?;
        }
        unsafe {
            let end = self.buf.as_ptr().add(self.head);
            std::ptr::write(end, value);
            self.head = (self.head + 1)%capacity;
        }
        self.len += 1;
        Ok(())
    }
    pub fn get(&mut self) -> Option<T> {
        let capacity = self.capacity();
        if self.head == self.tail && self.len != capacity {
            return None;
        }

        let get = unsafe { std::ptr::read(self.buf.as_ptr().add(self.tail)) };
        self.tail = (self.tail + 1)%capacity;
        self.len -= 1;

        Some(get)
    }
}

impl<T> Drop for CircularBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(self.buf.as_ptr().add(self.tail), self.len));
        }
    }
}