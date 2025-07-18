// pasm - src/shr/smallvec.rs
// --------------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    fmt::{Debug, Formatter},
    iter::Iterator,
    mem::MaybeUninit,
};

pub struct SmallVec<T, const N: usize> {
    pub len: usize,
    pub content: [MaybeUninit<T>; N],
}

impl<T, const N: usize> std::ops::Deref for SmallVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        use std::mem::transmute;
        unsafe { std::slice::from_raw_parts(transmute(self.content.as_ptr()), self.len) }
    }
}
impl<T, const N: usize> std::ops::DerefMut for SmallVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        use std::mem::transmute;
        unsafe { std::slice::from_raw_parts_mut(transmute(self.content.as_mut_ptr()), self.len) }
    }
}

impl<T, const N: usize> std::ops::Index<usize> for SmallVec<T, N> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx).expect("smv: index out of bounds")
    }
}

impl<T, const N: usize> Clone for SmallVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut b = [const { MaybeUninit::uninit() }; N];
        let mut idx = 0;
        for c in self.iter() {
            b[idx] = MaybeUninit::new(c.clone());
            idx += 1;
        }
        while idx != self.len() {
            b[idx] = MaybeUninit::uninit();
            idx += 1;
        }
        Self {
            len: self.len,
            content: b,
        }
    }
}

impl<T, const N: usize> Debug for SmallVec<T, N>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("[")?;
        for (i, e) in self.iter().enumerate() {
            e.fmt(f)?;
            if i + 1 != self.len() {
                f.write_str(", ")?;
            }
        }
        f.write_str("]")?;
        Ok(())
    }
}

impl<T, const N: usize> PartialEq for SmallVec<T, N>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        if self.len() != rhs.len() {
            return false;
        }
        for i in 0..self.len() {
            if self.get_unchecked(i) != rhs.get_unchecked(i) {
                return false;
            }
        }
        true
    }
}

impl<T, const N: usize> SmallVec<T, N> {
    pub fn clear(&mut self) {
        if std::mem::needs_drop::<T>() {
            for i in 0..self.len() {
                unsafe { std::ptr::drop_in_place(self.content[i].as_mut_ptr()) }
            }
        }
        self.len = 0;
    }
    pub const unsafe fn insert(&mut self, idx: usize, t: T) {
        self.content[idx] = MaybeUninit::new(t);
    }
    /// ATTENTION: use this function WITH ManuallyDrop<T>
    /// it is unsafe, because it allows for scenarios like:
    /// | ELEMENT | NONE | ELEMENT |
    /// which is just UB
    pub const unsafe fn take_owned(&mut self, idx: usize) -> Option<T> {
        if self.len() > idx {
            let element = self.content[idx].assume_init_read();
            self.content[idx] = MaybeUninit::uninit().assume_init();
            Some(element)
        } else {
            None
        }
    }
    /// ATTENTION: use this function WITH ManuallyDrop<T>
    /// it is unsafe, because it allows for scenarios like:
    /// | ELEMENT | NONE | ELEMENT |
    /// which is just UB
    pub const unsafe fn take_owned_unchecked(&mut self, idx: usize) -> T {
        let element = self.content[idx].assume_init_read();
        self.content[idx] = MaybeUninit::uninit().assume_init();
        element
    }
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            content: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
        }
    }
    pub const fn get_unchecked(&self, idx: usize) -> &T {
        unsafe { self.content[idx].assume_init_ref() }
    }
    #[inline]
    pub const fn push(&mut self, t: T) {
        self.content[self.len] = MaybeUninit::new(t);
        self.len += 1;
    }
    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            let s = unsafe { self.content[self.len()].assume_init_read() };
            self.content[self.len()] = unsafe { MaybeUninit::uninit().assume_init() };
            Some(s)
        }
    }
    #[inline]
    pub const fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len() {
            Some(unsafe { self.content[idx].assume_init_ref() })
        } else {
            None
        }
    }
    pub const fn can_push(&self) -> bool {
        self.len() < N
    }
    #[inline]
    pub const fn first(&self) -> Option<&T> {
        self.get(0)
    }
    #[inline]
    pub const fn last(&self) -> Option<&T> {
        self.get(self.len() - 1)
    }
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }
    pub fn into_vec(self) -> Vec<T> {
        (0..self.len())
            .map(|s| unsafe { self.content[s].assume_init_read() })
            .collect()
    }
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        (0..self.len()).map(move |s| unsafe { self.content[s].assume_init_read() })
    }
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len()).map(|s| unsafe { self.content[s].assume_init_ref() })
    }
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        (0..self.len()).map(|s| unsafe { &mut *self.content[s].as_mut_ptr() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut myvec: SmallVec<u8, 12> = SmallVec::new();
        myvec.push(10);
        assert_eq!(myvec.first(), Some(&10));
        myvec.push(20);
        assert_eq!(myvec.get(1), Some(&20));
        assert_eq!(myvec.len(), 2);
        let v = myvec.pop().unwrap();
        assert_eq!(v, 20);
        assert_eq!(myvec.len(), 1);
    }
}
