use std::{any::{Any, TypeId}, cell::{Ref, RefCell, UnsafeCell}, marker::PhantomData};

use parking_lot::RwLock;

use crate::routines::TypeIdMap;
pub struct UnsafeWorldCell<'w>(*mut World, PhantomData<(&'w World, &'w UnsafeCell<World>)>);
unsafe impl<'w> Send for UnsafeWorldCell<'w> {}
unsafe impl<'w> Sync for UnsafeWorldCell<'w> {}
impl<'w> UnsafeWorldCell<'w> {
    pub unsafe fn insert_resource<T: 'static>(&self, resource: T) {
        unsafe {
            let world = self.0.as_mut().unwrap();
            if let Some(resource) = world.component_ids.get(&TypeId::of::<T>()).cloned() {
                *world.resources.get_mut(resource).unwrap().borrow_mut() = Box::new(resource);
            } else {
                world.component_ids.insert(TypeId::of::<T>(), world.resources.len());
                world.resources.push(RefCell::new(Box::new(resource)));
            }
        }
    }
}
#[derive(Default)]
pub struct World {
    component_ids: TypeIdMap<usize>,
    pub resources: Vec<RefCell<Box<dyn Any>>>,
}
impl World {
    pub fn as_unsafe_world(&self) -> UnsafeWorldCell {
        UnsafeWorldCell(self as *const _ as *mut _, PhantomData)
    }
    pub fn get_resource<'a>(&'a self, id: &TypeId ) -> Option<Ref<'a, Box<dyn Any + 'static>>> {
        Some(self.resources[self.component_ids.get(id)?.clone()].borrow())
    }
    pub fn insert_resource<T: 'static>(&self, resource: T) {
        unsafe {
            self.as_unsafe_world().insert_resource(resource);
        }
    }
}
impl World {
    pub fn new() -> Self {
        Self::default()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}