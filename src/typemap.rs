use std::{any::TypeId, collections::HashMap, hash::{BuildHasher, Hasher}};

pub type TypeIdMap<T> = HashMap<TypeId, T, NopTypeIdBuildHasher>;

#[doc(hidden)]
#[derive(Default)]
pub struct NopTypeIdBuildHasher;

impl BuildHasher for NopTypeIdBuildHasher {
    type Hasher = NopTypeIdHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NopTypeIdHasher(0)
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct NopTypeIdHasher(u64);

impl Hasher for NopTypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i
    }
}
