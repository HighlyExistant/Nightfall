#![feature(allocator_api)]
use std::{alloc::{Allocator, Layout}, time::Instant};

use arena::{Arena, StandardArena, PtrArena};

mod arena;
fn main() {
    let benchmark = |allocator: &dyn Allocator, layout: Layout|{
        let bench = Instant::now();
        let alloc = allocator.allocate(layout).unwrap();
        unsafe { allocator.deallocate(alloc.cast(), layout) };
        println!("ARENA {:?}", bench.elapsed());
    };
    
    let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
    let arena = unsafe { StandardArena::new(1024*80) };
    
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    benchmark(&arena, Layout::new::<[u8;1024*80]>());
    let bench = Instant::now();
    let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
    unsafe { std::alloc::dealloc(alloc.cast(), Layout::new::<[u8;1024*80]>()) };
    println!("{:?}", bench.elapsed());
    let bench = Instant::now();
    let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
    unsafe { std::alloc::dealloc(alloc.cast(), Layout::new::<[u8;1024*80]>()) };
    println!("{:?}", bench.elapsed());
    let bench = Instant::now();
    let alloc = unsafe { std::alloc::alloc(Layout::new::<[u8;1024*80]>()) };
    unsafe { std::alloc::dealloc(alloc.cast(), Layout::new::<[u8;1024*80]>()) };
    println!("{:?}", bench.elapsed());
}
