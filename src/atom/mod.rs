#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom(u128);
impl Default for Atom {
    fn default() -> Self {
        Self::new()
    }
}
impl Atom {
    pub fn new() -> Self {
        static mut ATOM_COUNTER: u128 = 0;
        unsafe {
            if let Some(get) = ATOM_COUNTER.checked_add(1) {
                ATOM_COUNTER = get;
                Self(get)
            } else {
                panic!("How did this even happen!?!?");
            }
        }
    }
}