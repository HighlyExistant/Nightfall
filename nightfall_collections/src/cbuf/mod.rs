use std::{alloc::Layout, marker::PhantomData};

use anyhow::Ok;

use crate::error::CollectionError;
/// Fixed size circular buffer 
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    buf: Box<[T]>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(count: usize) -> Self {
        assert!(count > 1, "A circular buffer should not have a fixed size of 1, consider using just a regular buffer");
        let buf = unsafe {
            let data =  std::alloc::alloc_zeroed(Layout::array::<T>(count).unwrap()) as *mut T;
            let slice = std::slice::from_raw_parts_mut(data, count);
            Box::from_raw(slice)
        };
        Self { buf, head: 0, tail: 0, len: 0 }
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
        self.buf.len()
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
            let end = self.buf.as_mut_ptr().add(self.head);
            std::ptr::write(end, value);
            self.head = (self.head + 1)%capacity;
        }
        self.len += 1;
        Ok(())
    }
    pub fn get(&mut self) -> Option<&T> {
        let capacity = self.capacity();
        if self.head == self.tail && self.len != capacity {
            return None;
        }

        let get = &self.buf[self.tail];
        self.tail = (self.tail + 1)%capacity;
        self.len -= 1;

        Some(get)
    }
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let len = self.capacity();
        let next = (self.tail + 1)%len;
        if self.head == next {
            return None;
        }
        let get = &mut self.buf[next];
        self.tail = next;
        Some(get)
    }
}
