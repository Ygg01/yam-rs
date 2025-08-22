#![allow(unused)]

use core::ptr::copy;
use core::slice::ChunksExact;

const CHUNK_SIZE: usize = 64;
const EMPTY_CHUNK: [u8; CHUNK_SIZE] = [b' '; CHUNK_SIZE];

/// Docs
pub struct ChunkyIterWrap<'a> {
    iter: ChunksExact<'a, u8>,
}

impl<'a> Iterator for ChunkyIterWrap<'a> {
    type Item = &'a [u8; 64];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|chunk| unsafe { &*chunk.as_ptr().cast::<[u8; 64]>() })
    }
}

impl<'a> ChunkyIterWrap<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> ChunkyIterWrap<'a> {
        ChunkyIterWrap {
            iter: bytes.chunks_exact(CHUNK_SIZE),
        }
    }

    pub fn remaining_chunk(&self) -> [u8; CHUNK_SIZE] {
        let x = self.iter.remainder();
        let mut last_chunk = [b' '; CHUNK_SIZE];

        if x.len() < 64 {
            unsafe {
                copy(x.as_ptr(), last_chunk.as_mut_ptr(), x.len());
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
    let mut iter = ChunkyIterWrap::from_bytes(&x);
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
    let mut iter = ChunkyIterWrap::from_bytes(&x);
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
    let mut iter = ChunkyIterWrap::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));

    let mut c = [b' '; 64];
    c[0] = 1;
    c[1] = 1;
    c[2] = 1;
    assert_eq!(iter.next(), Some(&c));
    assert_eq!(iter.next(), None);
}
