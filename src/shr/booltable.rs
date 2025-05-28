// rasmx86_64 - src/shr/booltable.rs
// ---------------------------------
// made by matissoss
// licensed under MPL 2.0

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BoolTable16 {
    data: u16,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for BoolTable16 {
    fn to_string(&self) -> String {
        let mut idx = 0;
        let mut string = String::new();
        while idx < 16 {
            string.push_str(&self.get(idx).unwrap().to_string());
            idx += 1;
        }
        string
    }
}

impl Default for BoolTable16 {
    fn default() -> Self {
        Self::new()
    }
}

impl BoolTable16 {
    pub fn new() -> Self {
        Self { data: 0 }
    }
    // WARN: this function should only be used at initialization :)
    pub fn set(&mut self, idx: u8, bool: bool) {
        if !bool {
            return;
        }
        // math
        self.data |= (bool as u16) << idx
    }
    // WARN: this function should only be used at initialization :)
    // this one is for chaining
    pub fn setc(mut self, idx: u8, bool: bool) -> Self {
        self.set(idx, bool);
        self
    }
    pub fn get(&self, idx: u8) -> Option<bool> {
        if idx < 16 {
            Some((self.data >> idx) == 0x01)
        } else {
            None
        }
    }
}
