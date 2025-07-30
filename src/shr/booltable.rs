// pasm - src/shr/booltable.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoolTable8 {
    data: u8,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub const fn new() -> Self {
        Self { data: 0 }
    }
    pub const fn set(&mut self, idx: u8, bool: bool) {
        let mask = 0xFFFF ^ (0b1 << idx);
        self.data = (self.data & mask) | ((bool as u16) << idx)
    }
    // this one is for chaining
    pub const fn setc(mut self, idx: u8, bool: bool) -> Self {
        self.set(idx, bool);
        self
    }
    pub const fn at(&self, idx: u8) -> bool {
        let tmp = 0x01 << idx;
        self.data & tmp == tmp
    }
    pub const fn get(&self, idx: u8) -> Option<bool> {
        if idx < 16 {
            let tmp = 0x01 << idx;
            Some(self.data & tmp == tmp)
        } else {
            None
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for BoolTable8 {
    fn to_string(&self) -> String {
        let mut idx = 0;
        let mut string = String::new();
        while idx < 8 {
            string.push_str(&self.get(idx).unwrap().to_string());
            idx += 1;
        }
        string
    }
}

impl Default for BoolTable8 {
    fn default() -> Self {
        Self::new()
    }
}

impl BoolTable8 {
    pub const fn new() -> Self {
        Self { data: 0 }
    }
    pub const fn set(&mut self, idx: u8, bool: bool) {
        let mask = 0xFF ^ (0b1 << idx);
        self.data = (self.data & mask) | ((bool as u8) << idx)
    }
    // this one is for chaining
    pub const fn setc(mut self, idx: u8, bool: bool) -> Self {
        self.set(idx, bool);
        self
    }
    pub const fn at(&self, idx: u8) -> bool {
        let tmp = 0x01 << idx;
        self.data & tmp == tmp
    }
    pub fn get(&self, idx: u8) -> Option<bool> {
        if idx < 8 {
            let tmp = 0x01 << idx;
            Some(self.data & tmp == tmp)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn booltable_test() {
        let mut booltable = BoolTable16::new();
        booltable.set(1, true);
        assert!(booltable.data == 0b0000_0010);
        booltable.set(0, true);
        assert!(booltable.data == 0b0000_0011);
        assert!(booltable.get(0) == Some(true));
        let mut booltable = BoolTable8::new();
        booltable.set(1, true);
        assert!(booltable.data == 0b0000_0010);
        booltable.set(0, true);
        assert!(booltable.data == 0b0000_0011);
        assert!(booltable.get(0) == Some(true));
    }
}
