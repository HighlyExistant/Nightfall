#![allow(unused)]
use std::{alloc::{Allocator, Global}, cmp::Ordering, fmt::Display, marker::PhantomData};
// mod set;
mod map;
// pub use set::*;
pub use map::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
    Red,
    Black,
}
struct RBTreeNode<K: Ord, T> {
    color: Color,
    left: NodePtr<K, T>,
    right: NodePtr<K, T>,
    parent: NodePtr<K, T>,
    key: K,
    value: T
}
impl<K: Ord, T> RBTreeNode<K, T> {
    pub fn pair(&self) -> (K, T) {
        unsafe { (std::ptr::read(&self.key), std::ptr::read(&self.value)) }
    }
    pub fn pair_ref(&self) -> (&K, &T) {
        (&self.key, &self.value)
    }
}
#[derive(Debug)]
#[repr(transparent)]
pub struct NodePtr<K: Ord, T>(*mut RBTreeNode<K, T>);
impl<K: Ord, V> Clone for NodePtr<K, V> {
    fn clone(&self) -> NodePtr<K, V> {
        NodePtr(self.0)
    }
}
impl<K: Ord, V> Copy for NodePtr<K, V> {}

impl<K: Ord, V> Ord for NodePtr<K, V> {
    fn cmp(&self, other: &NodePtr<K, V>) -> Ordering {
        unsafe { (*self.0).key.cmp(&(*other.0).key) }
    }
}

impl<K: Ord, V> PartialOrd for NodePtr<K, V> {
    fn partial_cmp(&self, other: &NodePtr<K, V>) -> Option<Ordering> {
        unsafe { Some((*self.0).key.cmp(&(*other.0).key)) }
    }
}

impl<K: Ord, V> PartialEq for NodePtr<K, V> {
    fn eq(&self, other: &NodePtr<K, V>) -> bool {
        self.0 == other.0
    }
}
impl<K: Ord, V> Eq for NodePtr<K, V> {}

impl<K: Ord, T> NodePtr<K, T> {
    fn new(k: K, v: T) -> NodePtr<K, T> {
        let node = RBTreeNode {
            color: Color::Red,
            left: NodePtr::null(),
            right: NodePtr::null(),
            parent: NodePtr::null(),
            key: k,
            value: v,
        };
        NodePtr(Box::into_raw(Box::new(node)))
    }
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }
    fn node(&self) -> &RBTreeNode<K, T> {
        unsafe { &*self.0 }
    }
    fn node_mut(&self) -> &mut RBTreeNode<K, T> {
        unsafe { &mut *self.0 }
    }
    pub fn left(&self) -> NodePtr<K, T> {
        self.node().left
    }
    pub fn right(&self) -> NodePtr<K, T> {
        self.node().right
    }
    pub fn parent(&self) -> NodePtr<K, T> {
        self.node().parent
    }
    fn color(self) -> Color {
        self.node().color
    }
    fn set_color(&self, color: Color) {
        if self.is_null() {
            return;
        }
        self.node_mut().color = color;
    }
    pub fn set_left_child(&self, node: NodePtr<K, T>) {
        self.node_mut().left = node;
    }
    pub fn set_right_child(&self, node: NodePtr<K, T>) {
        self.node_mut().right = node;
    }
    pub fn set_parent(&self, node: NodePtr<K, T>) {
        self.node_mut().parent = node;
    }
    pub fn is_left_child(self) -> bool {
        self.parent().left() == self
    }
    pub fn is_right_child(self) -> bool {
        self.parent().right() == self
    }
    pub fn is_red_node(self) -> bool {
        if !self.is_null() {
            return self.node().color == Color::Red;
        }
        false
    }
    pub fn is_black_node(self) -> bool {
        if !self.is_null() {
            return self.node().color == Color::Black;
        }
        false
    }
    pub fn is_null(self) -> bool {
        self == Self::null()
    }
    pub fn key(&self) -> &K {
        &self.node().key
    }
    pub fn value(&self) -> &T {
        &self.node().value
    }
    pub fn value_mut(&self) -> &mut T {
        &mut self.node_mut().value
    }
    pub fn swap_color(&self, other: &Self) {
        std::mem::swap(&mut self.node_mut().color, &mut other.node_mut().color);
    }
}

pub struct RBTree<K: Ord, T, A: Allocator = Global> {
    root: NodePtr<K, T>,
    len: usize,
    alloc: A,
}
impl<K: Ord, T> RBTree<K, T>  {
    pub fn new() -> Self {
        Self::new_in(Global)
    }
}
impl<K: Ord, T, A: Allocator> RBTree<K, T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self { root: NodePtr::null(), len: 0, alloc }
    }
    pub fn insert(&mut self, key: K, value: T) {
        let mut node = NodePtr::new(key, value);
        let mut parent = NodePtr::null();
        let mut current = self.root;
        while !current.is_null() {
            parent = current;
            if node.key() < current.key() {
                current = current.left();
            } else {
                current = current.right();
            }
        }
        node.set_parent(parent);
        if parent.is_null() {
            self.root = node;
        } else if node.key() < parent.key() {
            parent.set_left_child(node);
        } else {
            parent.set_right_child(node);
        }
        self.len += 1;
        self.fix_insert(node);
    }
    
    pub fn get(&self, key: &K) -> Option<&T> {
        let mut node = self.find_node(key);
        if node.is_null() {
            None
        } else {
            unsafe { Some(std::mem::transmute(node.value())) }
        }
    }
    pub fn get_mut(&self, key: &K) -> Option<&mut T> {
        let mut node = self.find_node(&key);
        if node.is_null() {
            None
        } else {
            unsafe { Some(std::mem::transmute(node.value_mut())) }
        }
    }
    pub fn is_clear(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub(crate) fn find_node(&self, k: &K) -> NodePtr<K, T> {
        self.find_node_by(|p|p.cmp(k))
    }
    #[inline]
    pub(crate) fn find_node_by<F>(&self, mut f: F) -> NodePtr<K, T> 
        where F: FnMut(&K) -> Ordering {
        if self.root.is_null() {
            return NodePtr::null();
        }
        let mut temp = &self.root;
        unsafe {
            loop {
                let next = match f(&(*temp.0).key) {
                    Ordering::Greater => &mut (*temp.0).left,
                    Ordering::Less => &mut (*temp.0).right,
                    Ordering::Equal => return *temp,
                };
                if next.is_null() {
                    break;
                }
                temp = next;
            }
        }
        NodePtr::null()
    }
    #[inline]
    fn find_node_with_values_by<F>(&self, mut f: F) -> NodePtr<K, T> 
        where F: FnMut(&K, &T) -> Ordering {
        if self.root.is_null() {
            return NodePtr::null();
        }
        let mut temp = &self.root;
        unsafe {
            loop {
                let next = match f(&(*temp.0).key, &(*temp.0).value) {
                    Ordering::Greater => &mut (*temp.0).left,
                    Ordering::Less => &mut (*temp.0).right,
                    Ordering::Equal => return *temp,
                };
                if next.is_null() {
                    break;
                }
                temp = next;
            }
        }
        NodePtr::null()
    }
    fn delete(&mut self, z: NodePtr<K, T>) -> Option<T> {
        let mut x = NodePtr::<K, T>::null();
        let mut y = NodePtr::<K, T>::null();
        
        if z.is_null() {
            return None;
        }
        y = z;
        let mut y_original_color = y.color();
        if z.left().is_null() {
            x = z.right();
            self.transplant(z, z.right());
        } else if z.right().is_null() {
            x = z.left();
            self.transplant(z, z.left());
        } else {
            y = Self::minimum_node(z.right());
            y_original_color = y.color();
            x = y.right();
            if y.parent() == z {
                if !x.is_null() {
                    x.set_parent(y);
                }
            } else {
                self.transplant(y, y.right());
                y.set_right_child(z.right());
                y.right().set_parent(y);
            }
            self.transplant(z, y);
            y.set_left_child(z.left());
            y.left().set_parent(y);
            y.set_color(z.color());
        }
        let from_z = unsafe { Box::from_raw(z.0) };
        let val = unsafe { Some(std::ptr::read(&from_z.value)) };
        if y_original_color == Color::Black {
            self.fix_remove(x);
        }
        val
    }
    pub fn remove_by<F>(&mut self, f: F) -> Option<T> 
        where F: FnMut(&K) -> Ordering{
        let z = self.find_node_by(f);
        self.delete(z)
    }
    pub fn remove_wth_values_by<F>(&mut self, f: F) -> Option<T> 
        where F: FnMut(&K, &T) -> Ordering{
        let z = self.find_node_with_values_by(f);
        self.delete(z)
    }
    pub fn search_and_remove_by<F>(&mut self, f: F) -> Result<T, Option<T>> 
        where F: FnMut(&K) -> Ordering{
        match self.search_node_by(f) {
            Ok(o) => Ok(self.delete(o).unwrap()),
            Err(e) => Err(self.delete(e)),
        }
    }
    pub fn search_and_remove_with_values_by<F>(&mut self, f: F) -> Result<T, Option<T>> 
        where F: FnMut(&K, &T) -> Ordering{
        match self.search_node_with_values_by(f) {
            Ok(o) => Ok(self.delete(o).unwrap()),
            Err(e) => Err(self.delete(e)),
        }
    }
    pub fn search_and_remove(&mut self, k: &K) -> Result<T, Option<T>> {
        self.search_and_remove_by(|p|p.cmp(k))
    }

    pub fn remove(&mut self, key: &K) -> Option<T> {
        self.remove_by(|p|p.cmp(key))
    }
    /// Searches for the value inside a given key. If a key isn't found, it finds the next
    /// highest key and returns the value. If there is no highest key returns the last key found.
    pub fn search(&self, k: &K) -> Result<T, Option<T>> {
        self.search_by(|p| p.cmp(k))
    }
    pub fn search_by<F>(&self, mut f: F) -> Result<T, Option<T>> 
        where F: FnMut(&K) -> Ordering {
        self.search_node_by(f).map(|val|{
            unsafe { std::ptr::read(val.value()) }
        }).map_err(|val|{
            if val.is_null() {
                None
            } else {
                Some(unsafe { std::ptr::read(val.value()) })
            }
        })

    }
    pub fn search_node_by<F>(&self, mut f: F) -> Result<NodePtr<K, T>, NodePtr<K, T>> 
        where F: FnMut(&K) -> Ordering {
            if self.root.is_null() {
                return Err(NodePtr::null());
            }
            let mut temp = &self.root;
            let mut larger_than_value = NodePtr::<K, T>::null();
            unsafe {
                loop {
                    let next = match f(&(*temp.0).key) {
                        Ordering::Greater => { larger_than_value = *temp; &mut (*temp.0).left },
                        Ordering::Less => &mut (*temp.0).right,
                        Ordering::Equal => return Ok(*temp),
                    };
                    if next.is_null() {
                        return Err(larger_than_value);
                    }
                    temp = next;
                }
            }
    }
    pub fn search_node_with_values_by<F>(&self, mut f: F) -> Result<NodePtr<K, T>, NodePtr<K, T>> 
        where F: FnMut(&K, &T) -> Ordering {
            if self.root.is_null() {
                return Err(NodePtr::null());
            }
            let mut temp = &self.root;
            let mut larger_than_value = NodePtr::<K, T>::null();
            unsafe {
                loop {
                    let next = match f(&(*temp.0).key, &(*temp.0).value) {
                        Ordering::Greater => { larger_than_value = *temp; &mut (*temp.0).left },
                        Ordering::Less => &mut (*temp.0).right,
                        Ordering::Equal => return Ok(*temp),
                    };
                    if next.is_null() {
                        return Err(larger_than_value);
                    }
                    temp = next;
                }
            }
    }
    pub fn search_with_values_by<F>(&self, mut f: F) -> Result<T, Option<T>> 
        where F: FnMut(&K, &T) -> Ordering {
        self.search_node_with_values_by(f).map(|val|{
            unsafe { std::ptr::read(val.value()) }
        }).map_err(|val|{
            if val.is_null() {
                None
            } else {
                Some(unsafe { std::ptr::read(val.value()) })
            }
        })
    }
    pub fn iter(&self) -> RBTreeIterator<K, T, A> {
        RBTreeIterator::new(self)
    }
    pub fn iter_mut(&mut self) -> RBTreeIteratorMut<K, T, A> {
        RBTreeIteratorMut::new(self)
    }
    pub fn minimum(&self) -> &T {
        unsafe { std::mem::transmute(Self::minimum_node(self.root).value()) }
    }
    pub fn maximum(&self) -> &T {
        unsafe { std::mem::transmute(Self::maximum_node(self.root).value()) }
    }
    pub fn minimum_mut(&mut self) -> &mut T {
        unsafe { std::mem::transmute(Self::minimum_node(self.root).value_mut()) }
    }
    pub fn maximum_mut(&mut self) -> &mut T {
        unsafe { std::mem::transmute(Self::maximum_node(self.root).value_mut()) }
    }
    fn minimum_node(node: NodePtr<K, T>) -> NodePtr<K, T> {
        let mut current = node;
        while !current.left().is_null() {
            current = current.left();
        }
        current
    }
    fn maximum_node(node: NodePtr<K, T>) -> NodePtr<K, T> {
        let mut current = node;
        while !current.right().is_null() {
            current = current.right();
        }
        current
    }
    fn left_rotate(&mut self, x: NodePtr<K, T>) {
        let y = x.right();
        x.set_right_child(y.left());
        if !y.left().is_null() {
            y.left().set_parent(x);
        }
        y.set_parent(x.parent());
        if x.parent().is_null() {
            self.root = y;
        } else if x.is_left_child() {
            x.parent().set_left_child(y);
        } else {
            x.parent().set_right_child(y);
        }
        y.set_left_child(x);
        x.set_parent(y);
    }
    fn right_rotate(&mut self, x: NodePtr<K, T>) {
        let y = x.left();
        x.set_left_child(y.right());
        if !y.right().is_null() {
            y.right().set_parent(x);
        }
        y.set_parent(x.parent());
        if x.parent().is_null() {
            self.root = y;
        } else if x.is_right_child() {
            x.parent().set_right_child(y);
        } else {
            x.parent().set_left_child(y);
        }
        y.set_right_child(x);
        x.set_parent(y);
    }
    fn transplant(&mut self, u: NodePtr<K, T>, v: NodePtr<K, T>) {
        if u.parent().is_null() {
            self.root = v;
        } else if u.is_left_child() {
            u.parent().set_left_child(v);
        } else {
            u.parent().set_right_child(v);
        }
        if !v.is_null() {
            v.set_parent(u.parent());
        }
    }
    fn fix_insert(&mut self, mut node: NodePtr<K, T>) {
        let mut parent = NodePtr::null();
        let mut grandparent = NodePtr::null();
        while node != self.root && node.is_red_node()
            && node.parent().is_red_node() {
            parent = node.parent();
            grandparent = parent.parent();
            if parent.is_left_child() {
                let uncle = grandparent.right();
                if !uncle.is_null() 
                    && uncle.is_red_node() {
                    grandparent.set_color(Color::Red);
                    parent.set_color(Color::Black);
                    uncle.set_color(Color::Black);
                    node = grandparent;
                } else {
                    if node.is_right_child() {
                        self.left_rotate(parent);
                        node = parent;
                        parent = node.parent();
                    }
                    self.right_rotate(grandparent);
                    parent.swap_color(&grandparent);
                    node = parent;
                }
            } else {
                let uncle = grandparent.left();
                if !uncle.is_null() 
                    && uncle.is_red_node() {
                    grandparent.set_color(Color::Red);
                    parent.set_color(Color::Black);
                    uncle.set_color(Color::Black);
                    node = grandparent;
                } else {
                    if node.is_left_child() {
                        self.right_rotate(parent);
                        node = parent;
                        parent = node.parent()
                    }
                    self.left_rotate(grandparent);
                    parent.swap_color(&grandparent);
                    node = parent;
                }
            }
        }
        self.root.set_color(Color::Black);
    }
    fn fix_remove(&mut self, mut node: NodePtr<K, T>) {
        while node != self.root && node.is_black_node() {
            if node.is_left_child() {
                let mut sibling = node.parent().right();
                if sibling.is_red_node() {
                    sibling.set_color(Color::Black);
                    node.parent().set_color(Color::Red);
                    self.left_rotate(node.parent());
                    sibling = node.parent().right();
                }
                if sibling.left().is_black_node() && sibling.right().is_black_node() {
                    sibling.set_color(Color::Red);
                    node = node.parent();
                } else {
                    if sibling.right().is_black_node() {
                        if !sibling.left().is_null() {
                            sibling.left().set_color(Color::Black);
                        }
                        sibling.set_color(Color::Red);
                        self.right_rotate(sibling);
                        sibling = node.parent().right();
                    }
                    sibling.set_color(node.parent().color());
                    node.parent().set_color(Color::Black);
                    if !sibling.right().is_null() {
                        sibling.right().set_color(Color::Black);
                    }
                    self.left_rotate(node.parent());
                    node = self.root;
                }
            } else {
                let mut sibling = node.parent().left();
                if sibling.is_red_node() {
                    sibling.set_color(Color::Black);
                    node.parent().set_color(Color::Red);
                    self.right_rotate(node.parent());
                    sibling = node.parent().left();
                }
                if sibling.left().is_black_node() && sibling.right().is_black_node() {
                    sibling.set_color(Color::Red);
                    node = node.parent();
                } else {
                    if sibling.left().is_black_node() {
                        if !sibling.right().is_null() {
                            sibling.right().set_color(Color::Black);
                        }
                        sibling.set_color(Color::Red);
                        self.left_rotate(sibling);
                        sibling = node.parent().left();
                    }
                    sibling.set_color(node.parent().color());
                    node.parent().set_color(Color::Black);
                    if !sibling.left().is_null() {
                        sibling.left().set_color(Color::Black);
                    }
                    self.right_rotate(node.parent());
                    node = self.root;
                }
            }
        }
        self.root.set_color(Color::Black);
    }
    fn print_helper(&self, root: NodePtr<K, T>, indent: &mut String, last: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        where K: Display {
        if !root.is_null() {
            f.write_str(format!("{}", indent).as_str())?;
            if last {
                f.write_str("R----")?;
                indent.push_str("   ");
            } else {
                f.write_str("L----")?;
                indent.push_str("|  ");
            }
            let node = root;
            let color = if node.color() == Color::Red {
                "Red"
            } else {
                "Black"
            };
            f.write_str(format!("{} ({})\n", node.key(), color).as_str())?;
            let mut str1 = indent.clone();
            let mut str2 = indent.clone();
            self.print_helper(node.left(), &mut str1, false, f)?;
            self.print_helper(node.right(), &mut str2, true, f)?;
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        let mut iter = self.iter();
        while let Some(node) = iter.next_inner() {
            let from_node = unsafe { Box::from_raw(node.0) };
        }
        self.len = 0;
    }
}

impl<K: Ord, T, A: Allocator> Drop for RBTree<K, T, A> {
    fn drop(&mut self) {
        let mut iter = self.iter();
        while let Some(node) = iter.next_inner() {
            let from_node = unsafe { Box::from_raw(node.0) };
        }
    }
}

impl<K: Ord + Display, T> Display for RBTree<K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut indent = String::new();
        self.print_helper(self.root, &mut indent, true, f)
    }
}

pub struct RBTreeIterator<'a, K: Ord, T, A: Allocator> {
    root: &'a RBTree<K, T, A>,
    stack: Vec<NodePtr<K, T>>,
}

impl<'a, K: Ord, T, A: Allocator> RBTreeIterator<'a, K, T, A> {
    pub fn new(tree: &'a RBTree<K, T, A>) -> Self {
        let mut stack = Vec::with_capacity(tree.len);
        if !tree.root.is_null() {
            stack.push(tree.root);
            let mut current = tree.root.left();
            while !current.is_null() {
                stack.push(current);
                current = current.left();
            }
        }
        Self { root: tree, stack }
    }
    pub fn next_inner(&mut self) -> Option<NodePtr<K, T>> {
        if let Some(node) = self.stack.pop() {
            if !node.right().is_null() {
                self.stack.push(node.right());
                let mut current = node.right().left();
                while !current.is_null() {
                    self.stack.push(current);
                    current = current.left();
                }
            }
            Some(node)
        } else {
            None
        }
    }
}

impl<'a, K: Ord, T, A: Allocator> Iterator for RBTreeIterator<'a, K, T, A> {
    type Item = (&'a K, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Some(std::mem::transmute(self.next_inner()?.node().pair_ref())) }
    }
}


pub struct RBTreeIteratorMut<'a, K: Ord, T, A: Allocator> {
    root: &'a mut RBTree<K, T, A>,
    stack: Vec<NodePtr<K, T>>,
}

impl<'a, K: Ord, T, A: Allocator> RBTreeIteratorMut<'a, K, T, A> {
    pub fn new(tree: &'a mut RBTree<K, T, A>) -> Self {
        let mut stack = Vec::with_capacity(tree.len);
        if !tree.root.is_null() {
            stack.push(tree.root);
            let mut current = tree.root.left();
            while !current.is_null() {
                stack.push(current);
                current = current.left();
            }
        }
        Self { root: tree, stack }
    }
    pub fn next_inner(&mut self) -> Option<NodePtr<K, T>> {
        if let Some(node) = self.stack.pop() {
            if !node.right().is_null() {
                self.stack.push(node.right());
                let mut current = node.right().left();
                while !current.is_null() {
                    self.stack.push(current);
                    current = current.left();
                }
            }
            Some(node)
        } else {
            None
        }
    }
}

impl<'a, K: Ord, T, A: Allocator> Iterator for RBTreeIteratorMut<'a, K, T, A> {
    type Item = (&'a K, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Some(std::mem::transmute(self.next_inner()?.node().pair_ref())) }
    }
}

