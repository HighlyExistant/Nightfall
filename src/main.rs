use std::alloc::Layout;

use arena::{Arena, PtrArena};

mod arena;
fn main() {
    let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
    let arena = unsafe { PtrArena::from_raw(alloc, 1024*80) };
    let x = arena.allocate(Layout::new::<u8>()).unwrap();
    println!("Hello, world!");
}
