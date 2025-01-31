use std::alloc::{Allocator, Global};

use super::{RBTree, RBTreeIterator, RBTreeIteratorMut};

pub struct RBTreeMap<K: Ord, T, A: Allocator = Global> {
    base: RBTree<K, T, A>,
}
impl<K: Ord, T> RBTreeMap<K, T> {
    pub fn new() -> Self {
        Self { base: RBTree::new_in(Global) }
    }
}
impl<K: Ord, T, A: Allocator> RBTreeMap<K, T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self { base: RBTree::new_in(alloc) }
    }
    pub fn insert(&mut self, key: K, value: T) -> Option<T> {
        let node = self.base.find_node(&key);
        if node.is_null() {
            self.base.insert(key, value);
            None
        } else {
            let get = unsafe { std::ptr::read(node.value()) };
            node.node_mut().value = value;
            Some(get)
        }
    }
    pub fn get(&self, key: &K) -> Option<&T> {
        self.base.get(key)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        self.base.get_mut(key)
    }
    pub fn remove(&mut self, key: &K) -> Option<T> {
        self.base.remove(key)
    }
    pub fn iter(&self) -> RBTreeIterator<'_, K, T, A> {
        self.base.iter()
    }
    pub fn iter_mut(&mut self) -> RBTreeIteratorMut<'_, K, T, A> {
        self.base.iter_mut()
    }
    pub fn clear(&mut self) {
        self.base.clear();
    }
    pub fn is_clear(&self) -> bool {
        self.base.is_clear()
    }
}