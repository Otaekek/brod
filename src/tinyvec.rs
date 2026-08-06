use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::usize;

#[derive(Debug)]
pub struct TinyVec<T: Clone, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: u8,
}

impl<T: Clone, const N: usize> TinyVec<T, N> {
    pub fn new() -> Self {
        unsafe {
            Self {
                array: MaybeUninit::uninit().assume_init(),
                len: 0,
            }
        }
    }

    pub fn push(&mut self, n: T) {
        if self.len as usize >= N {
            panic!("Tinyvec overflow: {}", N);
        }
        unsafe {
            *self.array[self.len as usize].assume_init_mut() = n;
        }
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        } else {
            self.len -= 1;
            unsafe {
                return Some(self.array[self.len as usize].assume_init_read());
            }
        }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl<T: Clone, const N: usize> IndexMut<usize> for TinyVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            return self.array[index].assume_init_mut();
        }
    }
}

impl<T: Clone, const N: usize> Index<usize> for TinyVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            return self.array[index].assume_init_ref();
        }
    }
}

#[test]
fn test_tiny_vec() {
    let mut t = TinyVec::<usize, 8>::new();
    t.push(3);
    t.push(4);

    assert!(4 == t.pop().unwrap());
    assert!(3 == t.pop().unwrap());
    assert!(t.pop().is_none());
    assert!(t.pop().is_none());
    t.push(3);
    t.push(4);
    t.push(3);
    t.push(4);
    assert!(t.len() == 4);
}
