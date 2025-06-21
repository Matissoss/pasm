// rasmx86_64 - src/shr/smallvec.rs
// --------------------------------
// made by matissoss
// licensed under MPL 2.0

pub struct SmallVec<T, const N: usize> {
    len: usize,
    pub content: [T; N],
}

impl<T, const N: usize> SmallVec<T, N>
where
    T: Clone,
{
    #[inline]
    pub fn push(&mut self, t: T) {
        self.content[self.len] = t;
        self.len += 1;
    }
    #[inline]
    pub fn pop(&mut self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            Some(&self.content[self.len()])
        }
    }
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.content.get(idx)
    }
    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.get(0)
    }
    #[inline]
    pub fn last(&self) -> Option<&T> {
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
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.content[..self.len()].to_vec()
    }
    #[inline]
    pub fn iter(&self) -> &[T] {
        &self.content[..self.len()]
    }
    #[inline]
    pub fn iter_mut(&mut self) -> &mut [T] {
        &mut self.content[..self.len]
    }
    pub const fn blank(arr: [T; N]) -> Self {
        Self {
            len: 0,
            content: arr,
        }
    }
}

impl<T, const N: usize> SmallVec<T, N>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            content: std::array::from_fn(|_| T::default()),
        }
    }
}
impl<T, const N: usize> SmallVec<T, N>
where
    T: Default + Copy,
{
    pub fn new_copy() -> Self {
        Self {
            len: 0,
            content: [T::default(); N],
        }
    }
}

impl<T, const N: usize> Default for SmallVec<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut myvec: SmallVec<u8, 12> = SmallVec::new();
        myvec.push(10);
        println!("{:?}", myvec.content);
        assert_eq!(myvec.first(), Some(&10));
        myvec.push(20);
        assert_eq!(myvec.get(1), Some(&20));
        assert_eq!(myvec.len(), 2);
        let v = myvec.pop().unwrap();
        assert_eq!(v, &20);
        assert_eq!(myvec.len(), 1);
    }
}
