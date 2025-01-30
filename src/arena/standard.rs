use std::{alloc::{Allocator, Global, Layout}, marker::PhantomData, ptr::NonNull};

use super::{Arena, PtrArena};
pub struct NextArenaHeader {
    arena: Option<PtrArena>,
    next: Option<NonNull<NextArenaHeader>>,
}
pub struct StandardArena<A: Allocator> {
    arena: PtrArena,
    allocator: A,
}
impl StandardArena<Global> {
    pub fn new(size: usize) -> Self {
        Self::new_in(std::alloc::Global, size)
    }
}
impl<A: Allocator> StandardArena<A> {
    pub fn new_in(allocator: A, size: usize) -> Self {
        let arena = Self::allocate_arena(&allocator, size);
        Self { arena, allocator }
    }
    fn allocate_arena(allocator: &A, size: usize) -> PtrArena {
        let layout = Layout::from_size_align(size+std::mem::size_of::<NextArenaHeader>(), 1).unwrap();
        let allocation = allocator.allocate(layout).unwrap().as_ptr().cast::<u8>();
        let arena = unsafe { PtrArena::from_raw(allocation.add(std::mem::size_of::<NextArenaHeader>()), layout.size()-std::mem::size_of::<NextArenaHeader>()) };
        arena
    }
    fn get_arena_header(arena: &PtrArena) -> &mut NextArenaHeader {
        unsafe { arena.as_ptr().sub(std::mem::size_of::<NextArenaHeader>()).cast::<NextArenaHeader>().as_mut().unwrap() }
    }
}
impl<A: Allocator> Arena for StandardArena<A> {
    type Allocation = std::ptr::NonNull<[u8]>;
    fn arena_alloc(&self, layout: Layout) -> Option<Self::Allocation> {
        let mut current_arena = &self.arena;
        loop {
            if let Some(alloc) = current_arena.arena_alloc(layout) {
                return Some(alloc);   
            }
            // if we are here, arena allocation must've failed
            let header = Self::get_arena_header(current_arena);
            if let Some(arena) = &Self::get_arena_header(current_arena).arena {
                current_arena = arena;
            } else {
                // align the next allocation to a page and multiply it by 2 to 
                // double space before having to allocate another arena
                let arena = Self::allocate_arena(&self.allocator, (current_arena.size().next_multiple_of(4096)*2).max(layout.size()));
                header.arena = Some(arena);
            }
        }
    }
    fn allocated(&self) -> usize {
        let mut current_arena = Some(&self.arena);
        let mut allocated = 0;
        while let Some(arena) = current_arena {
            allocated += arena.allocated();
            current_arena = Self::get_arena_header(arena).arena.as_ref();
        }
        allocated
    }
    unsafe fn clear(&self) {
        let mut current_arena = Some(&self.arena);
        while let Some(arena) = current_arena {
            arena.clear();
            current_arena = Self::get_arena_header(arena).arena.as_ref();
        }
    }
    fn is_clear(&self) -> bool {
        let mut current_arena = &self.arena;
        if !current_arena.is_clear() {
            return false;
        }
        while let Some(arena) = &Self::get_arena_header(current_arena).arena {
            if !arena.is_clear() {
                return false;
            }
            current_arena = arena;
        }
        true
    }
    fn size(&self) -> usize {
        let mut current_arena = Some(&self.arena);
        let mut size = 0;
        while let Some(arena) = current_arena {
            size += arena.size();
            current_arena = Self::get_arena_header(arena).arena.as_ref();
        }
        size
    }
}

unsafe impl<A: Allocator> Allocator for StandardArena<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        <Self as Arena>::arena_alloc(self, layout).ok_or(std::alloc::AllocError)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // empty deallocate function, since we clear Arenas, not deallocate.
    }
}