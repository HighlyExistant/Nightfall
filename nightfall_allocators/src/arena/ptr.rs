#![allow(unused)]
use std::{alloc::{Allocator, Layout}, cell::Cell, rc::Rc, sync::Arc};

use crate::error::AllocError;
/// Represents an abstract arena allocator.
/// # Concepts
/// An arena allocator is useful for when you are going to allocate lots of scratch data 
/// that will be removed afterwards like a lifetime.
/// New empty arena of size 512
/// ```
/// ┌─────────────────────────────────────┐
/// │                512                  │
/// └─────────────────────────────────────┘
/// ```
/// Allocate size 12, Allocate size 50, Allocate size 32
/// ```
/// ┌────┬────┬────┬──────────────────────┐
/// │ 12 │ 50 │ 32 │         406          │
/// └────┴────┴────┴──────────────────────┘
/// ```
/// using `clear` resets the entire allocator, meaning you don't have to use the 
/// allocation to deallocate.
/// ```
/// ┌─────────────────────────────────────┐
/// │                512                  │
/// └─────────────────────────────────────┘
/// ```
/// One of the best use cases for an [`Arena`] is to use it as scrath memory.
/// Whenever you need lots of allocations, it's better to just use an [`Arena`]
/// as an allocation is really fast.
pub trait Arena {
    type Allocation;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation>;
    fn size(&self) -> usize;
    fn allocated(&self) -> usize;
    /// extremely dangerous function because calling this function and using a previously allocated value
    /// will result in that value accessing memory that a new allocation might currently own.
    /// There is also no way for the Arena to check whether previously allocated memory is being accessed again.
    unsafe fn clear(&self);
    fn is_clear(&self) -> bool ;
}

impl<T: Arena> Arena for Arc<T> {
    type Allocation = T::Allocation;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation> {
        (**self).arena_alloc(layout)
    }
    fn allocated(&self) -> usize {
        (**self).allocated()
    }
    fn size(&self) -> usize {
        (**self).size()
    }
    unsafe fn clear(&self) {
        unsafe {
            (**self).clear()
        }
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
impl<T: Arena> Arena for Rc<T> {
    type Allocation = T::Allocation;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation> {
        (**self).arena_alloc(layout)
    }
    fn allocated(&self) -> usize {
        (**self).allocated()
    }
    fn size(&self) -> usize {
        (**self).size()
    }
    unsafe fn clear(&self) {
        unsafe {
            (**self).clear()
        }
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
impl<T: Arena> Arena for Box<T> {
    type Allocation = T::Allocation;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation> {
        (**self).arena_alloc(layout)
    }
    fn allocated(&self) -> usize {
        (**self).allocated()
    }
    fn size(&self) -> usize {
        (**self).size()
    }
    unsafe fn clear(&self) {
        unsafe {
            (**self).clear()
        }
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
impl<T: Arena> Arena for &T {
    type Allocation = T::Allocation;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation> {
        (**self).arena_alloc(layout)
    }
    fn allocated(&self) -> usize {
        (**self).allocated()
    }
    fn size(&self) -> usize {
        (**self).size()
    }
    unsafe fn clear(&self) {
        unsafe {
            (**self).clear()
        }
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
/// Arena allocator that uses a pointer and a size to represent allocations.
/// Used when performance is preffered.
#[derive(Clone, Debug)]
pub struct PtrArena {
    ptr: *mut u8,
    size: usize,
    offset: Cell<usize>,
}
impl PartialEq for PtrArena {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.eq(&other.ptr)
    }
}
impl Eq for PtrArena {}

impl PtrArena {
    pub unsafe fn from_raw(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size, offset: Cell::new(0) }
    }
    pub unsafe fn from_slice(slice: &mut [u8]) -> Self {
        Self { ptr: slice.as_mut_ptr(), size: slice.len(), offset: Cell::new(0) }
    }
    pub fn from_arena(arena: &dyn Arena<Allocation = *mut u8>, layout: Layout) -> anyhow::Result<Self> {
        let ptr = arena.arena_alloc(layout)?;
        Ok(Self { ptr, size: layout.size(), offset: Cell::new(0) })
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl Arena for PtrArena {
    type Allocation = std::ptr::NonNull<[u8]>;
    fn arena_alloc(&self, layout: Layout) -> anyhow::Result<Self::Allocation> {
        self.offset.set(self.offset.get().next_multiple_of(layout.align())); // align type
        let offset = self.offset.get();
        if let Some(new_offset) = offset.checked_add(layout.size()) { // checks for addition overflow, allocation can not overflow
            if new_offset > self.size { // allocation too larg
                Err(AllocError::OutOfMemory)?
            }
            self.offset.set(new_offset);
            unsafe { Ok(std::ptr::NonNull::new(std::slice::from_raw_parts_mut(self.ptr.add(offset), layout.size())).unwrap()) }
        } else { // size too large, not enough space
            Err(AllocError::OutOfMemory)?
        }
    }
    fn size(&self) -> usize {
        self.size
    }
    fn allocated(&self) -> usize {
        self.offset.get()
    }
    unsafe fn clear(&self) {
        self.offset.set(0);
    }
    fn is_clear(&self) -> bool {
        self.offset.get() == 0
    }
}
unsafe impl Allocator for PtrArena {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        <Self as Arena>::arena_alloc(self, layout).map_err(|_|std::alloc::AllocError)
    }
    unsafe fn deallocate(&self, _: std::ptr::NonNull<u8>, _: Layout) {
        // empty deallocate function, since we clear Arenas, not deallocate.
    }
}

#[cfg(test)]
mod test {
    use std::{alloc::{Allocator, Layout}, ops::Sub, time::Instant};

    use super::{Arena, PtrArena};
    #[test]
    fn arena_test() {
        let allocation = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
        let arena = unsafe { PtrArena::from_raw(allocation, 1024*80) };
        let arena_alloc1 = arena.arena_alloc(Layout::new::<u8>()).unwrap(); // size of u8 = 1
        assert!(arena.allocated() == 1, "Test if the arena correctly allocated a u8");
        let arena_alloc2 = arena.arena_alloc(Layout::new::<u64>()).unwrap(); // size of u64 = 8 align = 8
        assert!(arena.allocated() == 16, "Test to see whether it was correctly aligned and allocated");
        assert!((unsafe { arena_alloc2.as_ref().as_ptr() } as usize).sub(unsafe { arena_alloc1.as_ref().as_ptr() } as usize) == 8, "testing to see if the location was correctly allocated");
    }
    #[test]
    fn allocator_test() {
        let allocation = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
        let arena = unsafe { PtrArena::from_raw(allocation, 1024*80) };
        let mut arena_vector = Vec::<u8, &PtrArena>::new_in(&arena);
        for i in 0..10 {
            arena_vector.push(i);
        }
        assert!(arena_vector == vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "Testing vectors allocate correctly");
        let mut boxed = Box::new_in(42, &arena);
        assert!(*boxed == 42, "Testing box allocated correctly");
    }
    fn allocate_all() {
        let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
        let arena = unsafe { PtrArena::from_raw(alloc, 1024*80) };
        let alloc = arena.allocate(Layout::new::<[u8;1024*80]>()).unwrap();
        unsafe { arena.clear() };
        assert!(arena.allocated() == 0, "Testing if clear works correctly");
        let alloc = arena.allocate(Layout::new::<[u8;1024*80]>()).unwrap();
        assert!(arena.allocated() == 1024*80, "Testing to see if reallocation works");
    }
}