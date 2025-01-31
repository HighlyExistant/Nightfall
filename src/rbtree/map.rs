use std::alloc::{Allocator, Global};

use super::{Iter, IterMut, RBTree, Values, ValuesMut};

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
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &T)> {
        self.base.get_key_value(key)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        self.base.get_mut(key)
    }
    pub fn remove(&mut self, key: &K) -> Option<T> {
        self.base.remove(key)
    }
    pub fn iter(&self) -> Iter<'_, K, T, A> {
        self.base.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, K, T, A> {
        self.base.iter_mut()
    }
    pub fn values(&self) -> Values<K, T, A> {
        Values { iter: self.iter() }
    }
    pub fn values_mut(&mut self) -> ValuesMut<K, T, A> {
        ValuesMut { iter: self.iter_mut() }
    }
    pub fn clear(&mut self) {
        self.base.clear();
    }
    pub(crate) fn find_node(&self, key: &K) -> super::NodePtr<K, T> {
        self.base.find_node(key)
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