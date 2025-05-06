use std::{alloc::{Allocator, Global}, iter::Chain};

use super::{RBTree, RBTreeMap};

pub struct RBTreeSet<K: Ord, A: Allocator = Global> {
    base: RBTreeMap<K, (), A>,
}
impl<K: Ord> RBTreeSet<K> {
    pub fn new() -> Self {
        Self::new_in(Global)
    }
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
    pub fn iter(&self) -> Iter<'_, K, A> {
        Iter { iter: self.base.keys() }
    }
    pub fn difference<'a>(&'a self, other: &'a RBTreeSet<K, A>) -> Difference<'a, K, A> {
        Difference { iter: self.iter(), other }
    }
    pub fn union<'a>(&'a self, other: &'a RBTreeSet<K, A>) -> Union<'a, K, A> {
        if self.len() >= other.len() {
            Union { iter: self.iter().chain(other.difference(self)) }
        } else {
            Union { iter: other.iter().chain(self.difference(other)) }
        }
    }
    pub fn intersection<'a>(&'a self, other: &'a RBTreeSet<K, A>) -> Intersection<'a, K, A> {
        if self.len() <= other.len() {
            Intersection { iter: self.iter(), other }
        } else {
            Intersection { iter: other.iter(), other: self }
        }
    }
    /// Iterator over the symmetric difference of
    /// `self` and `other`. These are the values in `self` or
    /// in `other` but not in both.
    /// 
    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a RBTreeSet<K, A>,
    ) -> SymmetricDifference<'a, K, A> {
        SymmetricDifference { iter: self.difference(other).chain(other.difference(self)) }
    }
    pub fn clear(&mut self) {
        self.base.clear();
    }
    pub fn is_clear(&self) -> bool {
        self.base.is_clear()
    }
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K: Ord, A: Allocator> Extend<K> for RBTreeSet<K, A> {
    fn extend<T: IntoIterator<Item = K>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for key in iter {
            self.base.insert(key, ());
        }
    }
}
impl<K: Ord, const N: usize> From<[K; N]> for RBTreeSet<K> {
    fn from(value: [K; N]) -> Self {
        let mut base = Self::new();
        base.extend(value);
        base
    }
}

#[repr(transparent)]
pub struct Iter<'a, K: Ord, A: Allocator> {
    iter: super::Keys<'a, K, (), A>,
}

impl<'a, K: Ord, A: Allocator> Iterator for Iter<'a, K, A> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct Difference<'a, K: Ord, A: Allocator> {
    iter: Iter<'a, K, A>,
    other: &'a RBTreeSet<K, A>,
}

impl<'a, K: Ord, A: Allocator> Iterator for Difference<'a, K, A> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let elt = self.iter.next()?;
            if !self.other.contains(elt) {
                return Some(elt);
            }
        }
    }
}
pub struct Union<'a, K: Ord + 'a, A: Allocator + 'a> {
    iter: Chain<Iter<'a, K, A>, Difference<'a, K, A>>,
}

impl<'a, K: Ord, A: Allocator> Iterator for Union<'a, K, A> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}
pub struct Intersection<'a, K: Ord + 'a, A: Allocator + 'a> {
    // iterator of the first set
    iter: Iter<'a, K, A>,
    // the second set
    other: &'a RBTreeSet<K, A>,
}

impl<'a, K: Ord + 'a, A: Allocator + 'a> Iterator for Intersection<'a, K, A>{
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        loop {
            let elt = self.iter.next()?;
            if self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, |acc, elt| if self.other.contains(elt) { f(acc, elt) } else { acc })
    }
}

pub struct SymmetricDifference<'a, K: Ord + 'a, A: Allocator + 'a> {
    iter: Chain<Difference<'a, K, A>, Difference<'a, K, A>>,
}
impl<'a, K: Ord + 'a, A: Allocator + 'a> Iterator for SymmetricDifference<'a, K, A> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}