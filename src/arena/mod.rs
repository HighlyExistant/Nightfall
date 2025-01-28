use std::{alloc::Layout, cell::Cell, ops::Add, pin::Pin, rc::Rc, sync::Arc};
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
pub trait Arena {
    type Allocation;
    fn allocate(&self, layout: Layout) -> Option<Self::Allocation>;
    fn clear(&self);
    fn is_clear(&self) -> bool ;
}

impl<T: Arena> Arena for Arc<T> {
    type Allocation = T::Allocation;
    fn allocate(&self, layout: Layout) -> Option<Self::Allocation> {
        (**self).allocate(layout)
    }
    fn clear(&self) {
        (**self).clear()
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
impl<T: Arena> Arena for Rc<T> {
    type Allocation = T::Allocation;
    fn allocate(&self, layout: Layout) -> Option<Self::Allocation> {
        (**self).allocate(layout)
    }
    fn clear(&self) {
        (**self).clear()
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
impl<T: Arena> Arena for Box<T> {
    type Allocation = T::Allocation;
    fn allocate(&self, layout: Layout) -> Option<Self::Allocation> {
        (**self).allocate(layout)
    }
    fn clear(&self) {
        (**self).clear()
    }
    fn is_clear(&self) -> bool {
        (**self).is_clear()
    }
}
/// Arena allocator that uses a pointer and a size to represent allocations.
/// Used when performance is preffered
pub struct PtrArena {
    ptr: *mut u8,
    size: usize,
    offset: Cell<usize>,
}

impl PtrArena {
    pub fn from_raw(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size, offset: Cell::new(0) }
    }
    pub fn from_slice(slice: &mut [u8]) -> Self {
        Self { ptr: slice.as_mut_ptr(), size: slice.len(), offset: Cell::new(0) }
    }
    pub fn from_arena(arena: &dyn Arena<Allocation = *mut u8>, layout: Layout) -> Option<Self> {
        let ptr = arena.allocate(layout)?;
        Some(Self { ptr, size: layout.size(), offset: Cell::new(0) })
    }
}

impl Arena for PtrArena {
    type Allocation = *mut u8;
    fn allocate(&self, layout: Layout) -> Option<Self::Allocation> {
        self.offset.set(self.offset.get().next_multiple_of(layout.align())); // align type
        let offset = self.offset.get();
        if let Some(new_offset) = offset.checked_add(layout.size()) { // checks for addition overflow, allocation can not overflow
            if new_offset > self.size { // allocation too larg
                return None;
            }
            self.offset.set(new_offset);
            unsafe { Some(self.ptr.add(offset)) }
        } else { // size too large, not enough space
            None
        }
    }
    fn clear(&self) {
        self.offset.set(0);
    }
    fn is_clear(&self) -> bool {
        self.offset.get() == 0
    }
}