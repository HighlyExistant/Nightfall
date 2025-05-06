use nightfall_allocators::arena::StandardArena;
use nightfall_collections::{bitmap::Bitmap, CircularBuffer};

fn main() {
    let mut bmap = Bitmap::new(20, 20);
    bmap.flip(10, 19, true);
}