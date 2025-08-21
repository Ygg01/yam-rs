#![allow(unused)]

use core::intrinsics::copy;

const EMPTY_CHUNK: [u8; 64] = [b' '; 64];

pub struct ChunkyIterator<'a> {
    bytes: &'a [u8],
}

const CHUNK_SIZE: usize = 64;

impl<'a> Iterator for ChunkyIterator<'a> {
    type Item = &'a [u8; 64];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8; 64]> {
        if self.bytes.len() < CHUNK_SIZE {
            None
        } else {
            let (first, second) = self.bytes.split_at(CHUNK_SIZE);
            self.bytes = second;
            Some(unsafe { &*first.as_ptr().cast::<[u8; 64]>() })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.bytes.len() / CHUNK_SIZE;
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl<'a> ExactSizeIterator for ChunkyIterator<'a> {}

impl ChunkyIterator<'_> {
    pub fn from_bytes(bytes: &[u8]) -> ChunkyIterator {
        ChunkyIterator { bytes }
    }

    pub fn remaining_chunk(&self) -> [u8; 64] {
        let mut last_chunk = [b' '; 64];
        if self.bytes.len() < 64 {
            unsafe {
                copy(
                    self.bytes.as_ptr(),
                    last_chunk.as_mut_ptr(),
                    self.bytes.len(),
                );
            }
        }

        last_chunk
    }
}

#[test]
fn test_chunk() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let x = [a, b].concat();
    let mut iter = ChunkyIterator::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), Some(&b));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_chunk_rem() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let mut x = [a, b].concat();
    x.push(3);
    let mut iter = ChunkyIterator::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), Some(&b));

    let mut c = [b' '; 64];
    c[0] = 3;
    assert_eq!(iter.next(), Some(&c));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_chunk_rem_minus() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let mut x = [a, b].concat();
    x.drain(67..);
    let mut iter = ChunkyIterator::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));

    let mut c = [b' '; 64];
    c[0] = 1;
    c[1] = 1;
    c[2] = 1;
    assert_eq!(iter.next(), Some(&c));
    assert_eq!(iter.next(), None);
}
