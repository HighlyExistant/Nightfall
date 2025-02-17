use std::{alloc::{Allocator, Global}, ops::Deref, sync::Arc};

use thread_local::ThreadLocal;

use super::{Arena, StandardArena};
#[repr(transparent)]
struct SendSyncStandardArena<A: Allocator + Send + Sync>(StandardArena<A>);
impl<A: Allocator + Send + Sync> Deref for SendSyncStandardArena<A> {
    type Target = StandardArena<A>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
unsafe impl<A: Allocator + Send + Sync> Send for SendSyncStandardArena<A> {}
unsafe impl<A: Allocator + Send + Sync> Sync for SendSyncStandardArena<A> {}
/// A thread safe arena allocator that by making a new arena per thread.
/// It's much better to use only a single global AsyncArena than having multiple.
pub struct AsyncArena<A: Allocator + Send + Sync + Clone = Global> {
    thread_local: ThreadLocal<SendSyncStandardArena<A>>,
    start_size: usize,
    alloc: A,
}
impl AsyncArena {
    pub fn new(start_size: usize) -> Arc<Self> {
        // align start_size to page size
        Arc::new(Self { thread_local: ThreadLocal::new(), start_size: start_size.next_multiple_of(4096), alloc: Global })
    }
}

impl<A: Allocator + Send + Sync + Clone> AsyncArena<A> {
    pub fn new_in(alloc: A, start_size: usize) -> Self {
        Self { thread_local: ThreadLocal::new(), start_size, alloc }
    }
    fn get_arena(&self) -> &SendSyncStandardArena<A> {
        self.thread_local.get_or(||{
            SendSyncStandardArena(StandardArena::new_in(self.alloc.clone(), self.start_size))
        })
    }
}

impl<A: Allocator + Send + Sync + Clone> Arena for  AsyncArena<A> {
    type Allocation = std::ptr::NonNull<[u8]>;
    fn arena_alloc(&self, layout: std::alloc::Layout) -> anyhow::Result<Self::Allocation> {
        self.get_arena().arena_alloc(layout)
    }
    fn allocated(&self) -> usize {
        self.get_arena().allocated()
    }
    unsafe fn clear(&self) {
        self.get_arena().clear()
    }
    fn is_clear(&self) -> bool {
        self.get_arena().is_clear()
    }
    fn size(&self) -> usize {
        self.get_arena().size()
    }
}