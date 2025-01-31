use std::alloc::Allocator;

use super::RBTree;

pub struct RBTreeSet<K: Ord, A: Allocator> {
    tree: RBTree<K, (), A>,
}

impl<K: Ord, A: Allocator> RBTreeSet<K, A> {
    
}