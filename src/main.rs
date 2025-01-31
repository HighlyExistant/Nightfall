#![feature(allocator_api)]
use std::{alloc::{Allocator, Global, Layout}, collections::{BTreeMap, HashMap, HashSet}, time::{Duration, Instant}};

use arena::{Arena, StandardArena, PtrArena};
use rbtree::RBTreeMap;

mod arena;
mod rbtree;
macro_rules! benchmark {
    ($code:block, $msg:expr) => {
        {
            let ___benchmark___ = std::time::Instant::now();
            $code
            let __ret__ = ___benchmark___.elapsed();
            println!("{} {:?}", $msg, __ret__);
            __ret__
        }
    };
}
macro_rules! benchmark_multiple {
    ($code:block, $msg:expr, $range:expr) => {
        {
            let mut sum = Duration::from_secs(0);
            let mut count = 0;
            for _ in $range {
                let ___benchmark___ = std::time::Instant::now();
                $code
                sum += ___benchmark___.elapsed();
                count += 1;
            }
            let amount = Duration::from_secs_f64(sum.as_secs_f64()/count as f64);
            println!("{} {:?}", $msg, amount);
        }
    };
}
fn main() {
    let mut arena = StandardArena::new(1024*1024*8);
    let mut hash = BTreeMap::<usize, usize>::new();
    let mut rbtree = RBTreeMap::<usize, usize>::new();
    let x = Duration::from_secs(0).as_secs_f64();
    benchmark!({
        for i in 0..100000 {
            hash.insert(rand::random_range(0..100000), rand::random_range(0..100000));
        }
    }, "Hashmap insert");
    benchmark!({
        for i in 0..100000 {
            rbtree.insert(rand::random_range(0..100000), rand::random_range(0..100000));
        }
    }, "RBTree insert");
    
    benchmark!({
        for i in 0..100000 {
            let x = hash.get(&rand::random_range(0..100000));
        }
    }, "Hashmap get");
    benchmark!({
        for i in 0..100000 {
            let x = rbtree.get(&rand::random_range(0..100000));
        }
    }, "RBTree get");
    
    benchmark_multiple!({
        for i in hash.iter() {
            let x = i;
        }
    }, "Hashmap iter", 0..100000);
    benchmark_multiple!({
        for i in rbtree.iter() {
            let x = i;
        }
    }, "RBTree iter", 0..100000);
    let map = HashMap::<usize, usize>::new();
    let set = HashSet::<usize>::new();
    let x = map.get(&0);
    map.values();
    // before optimization
    // Hashmap iter 2.977001ms
    // RBTree iter 3.359479ms
    // after optimization
    // Hashmap iter 2.958879ms
    // RBTree iter 3.318349ms
}
