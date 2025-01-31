use std::alloc::Allocator;

use super::{RBTree, RBTreeMap};

pub struct RBTreeSet<K: Ord, A: Allocator> {
    base: RBTreeMap<K, (), A>,
}

impl<K: Ord, A: Allocator> RBTreeSet<K, A> {
    pub fn new_in(alloc: A) -> Self {
        Self { base: RBTreeMap::new_in(alloc) }
    }
    pub fn insert(&mut self, key: K) -> bool {
        self.base.insert(key, ()).is_some()
    }
    pub fn get(&self, key: &K) -> Option<&K> {
        self.base.get_key_value(key).map(|(k,t)|k)
    }
    pub fn remove(&mut self, key: &K) -> bool {
        self.base.remove(key).is_some()
    }
    pub fn iter(&self) -> Iter<'_, K, (), A> {
        Iter { iter: self.base.values() }
    }
    pub fn clear(&mut self) {
        self.base.clear();
    }
    pub fn is_clear(&self) -> bool {
        self.base.is_clear()
    }
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.base.len()
    }
}
#[repr(transparent)]
pub struct Iter<'a, K: Ord, T, A: Allocator> {
    iter: super::Values<'a, K, T, A>,
}

impl<'a, K: Ord, T, A: Allocator> Iterator for Iter<'a, K, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}