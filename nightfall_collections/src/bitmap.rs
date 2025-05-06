use std::ops::Div;

pub struct Bitmap {
    width: u32,
    height: u32,
    buf: Vec<u64>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width*height).div_ceil(u64::BITS) as usize;
        Self { width, height, buf: vec![0; size] }
    }
    pub fn flip(&mut self, x: u32, y: u32, flip: bool) {
        assert!(x < self.width && y < self.height, "coordinates out of bounds for Bitmap");
        let coords = x+(y*self.width);
        let idx = coords.div(u64::BITS) as usize;
        let bit = coords%u64::BITS;
        if flip {
            self.buf[idx] |= 1 << bit;
        } else {
            self.buf[idx] &= !(1 << bit);
        }
    }
    pub fn active(&self, x: u32, y: u32) -> bool {
        assert!(x < self.width && y < self.height, "coordinates out of bounds for Bitmap");
        let coords = x+(y*self.width);
        let idx = coords.div(u64::BITS) as usize;
        let bit = coords%u64::BITS;
        self.buf[idx] & (1 << bit) != 0
    }
}