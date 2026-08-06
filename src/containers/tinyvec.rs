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
        self.array[self.len as usize].write(n);
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

impl<T: Clone, const N: usize> Drop for TinyVec<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
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

#[test]
fn test_len_starts_at_zero() {
    let t = TinyVec::<usize, 8>::new();
    assert_eq!(t.len(), 0);
}

#[test]
fn test_len_tracks_push_and_pop() {
    let mut t = TinyVec::<usize, 8>::new();
    assert_eq!(t.len(), 0);
    t.push(1);
    assert_eq!(t.len(), 1);
    t.push(2);
    t.push(3);
    assert_eq!(t.len(), 3);
    t.pop();
    assert_eq!(t.len(), 2);
    t.pop();
    t.pop();
    assert_eq!(t.len(), 0);
}

#[test]
fn test_pop_order_is_lifo() {
    let mut t = TinyVec::<i32, 8>::new();
    for i in 0..8 {
        t.push(i);
    }
    for i in (0..8).rev() {
        assert_eq!(t.pop(), Some(i));
    }
    assert_eq!(t.pop(), None);
}

#[test]
fn test_fill_to_exact_capacity() {
    let mut t = TinyVec::<u8, 4>::new();
    t.push(10);
    t.push(20);
    t.push(30);
    t.push(40);
    assert_eq!(t.len(), 4);
    assert_eq!(t.pop(), Some(40));
    assert_eq!(t.pop(), Some(30));
    assert_eq!(t.pop(), Some(20));
    assert_eq!(t.pop(), Some(10));
    assert_eq!(t.pop(), None);
}

#[test]
#[should_panic(expected = "Tinyvec overflow")]
fn test_push_past_capacity_panics() {
    let mut t = TinyVec::<u8, 2>::new();
    t.push(1);
    t.push(2);
    t.push(3);
}

#[test]
fn test_empty_refill_cycle() {
    let mut t = TinyVec::<usize, 3>::new();
    for round in 0..5 {
        t.push(round);
        t.push(round + 1);
        assert_eq!(t.len(), 2);
        assert_eq!(t.pop(), Some(round + 1));
        assert_eq!(t.pop(), Some(round));
        assert_eq!(t.pop(), None);
        assert_eq!(t.len(), 0);
    }
}

#[test]
fn test_index_read_only_within_len() {
    let mut t = TinyVec::<i32, 8>::new();
    t.push(100);
    t.push(200);
    t.push(300);
    assert_eq!(t[0], 100);
    assert_eq!(t[1], 200);
    assert_eq!(t[2], 300);
}

#[test]
fn test_index_mut_overwrites_in_place() {
    let mut t = TinyVec::<i32, 8>::new();
    t.push(1);
    t.push(2);
    t.push(3);
    t[1] = 42;
    assert_eq!(t[0], 1);
    assert_eq!(t[1], 42);
    assert_eq!(t[2], 3);
    assert_eq!(t.pop(), Some(3));
    assert_eq!(t.pop(), Some(42));
    assert_eq!(t.pop(), Some(1));
}

#[test]
fn test_single_element_capacity() {
    let mut t = TinyVec::<u8, 1>::new();
    assert_eq!(t.pop(), None);
    t.push(7);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0], 7);
    assert_eq!(t.pop(), Some(7));
    assert_eq!(t.pop(), None);
    t.push(9);
    assert_eq!(t.pop(), Some(9));
}

#[test]
fn test_drop_type_refill() {
    let mut t = TinyVec::<String, 4>::new();
    t.push("hello".to_string());
    t.push("world".to_string());
    assert_eq!(t.pop(), Some("world".to_string()));
    assert_eq!(t.pop(), Some("hello".to_string()));
    t.push("again".to_string());
    assert_eq!(t.pop(), Some("again".to_string()));
}

#[test]
fn test_drop_type_dropped_without_popping() {
    let mut t = TinyVec::<String, 4>::new();
    t.push("one".to_string());
    t.push("two".to_string());
}

#[test]
fn test_index_mut_drops_old_value() {
    let mut t = TinyVec::<String, 4>::new();
    t.push("old".to_string());
    t[0] = "new".to_string();
    assert_eq!(t.pop(), Some("new".to_string()));
}
