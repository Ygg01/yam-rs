#![allow(unused)]

use alloc::vec::Vec;
use core::ptr;
use core::slice::from_raw_parts;

const EMPTY_CHUNK: [u8; 64] = [b' '; 64];

pub struct ChunkyIterator<'a> {
    bytes: &'a [u8],
    extra_bytes: Vec<u8>,
}

impl<'a> Iterator for ChunkyIterator<'a> {
    type Item = &'a [u8; 64];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.len() {
            i if i >= 64 => {
                let len = self.bytes.len();
                let ptr = self.bytes.as_ptr();
                // SAFETY:
                // From raw parts it is safe
                // We manually verified the bounds of the split.
                let (first, tail) = unsafe {
                    (
                        from_raw_parts(ptr, 64),
                        from_raw_parts(ptr.add(64), len - 64),
                    )
                };
                self.bytes = tail;
                // SAFETY: We explicitly check for the correct number of elements,
                //   and do not let the references outlive the slice.
                Some(unsafe { &*first.as_ptr().cast::<[u8; 64]>() })
            }
            i if i > 0 && i < 64 => unsafe {
                // SAFETY: We pad the len to 64
                // First copy 64 spaces
                // Then copy over what remains of the data.
                // In theory, spaces don't affect YAML parsing if no entry is present.
                self.extra_bytes.set_len(64);
                ptr::copy_nonoverlapping(EMPTY_CHUNK.as_ptr(), self.extra_bytes.as_mut_ptr(), 64);
                ptr::copy_nonoverlapping(
                    self.bytes.as_ptr(),
                    self.extra_bytes.as_mut_ptr(),
                    self.bytes.len(),
                );
                self.bytes = &[];
                Some(&*self.extra_bytes.as_ptr().cast::<[u8; 64]>())
            },
            _ => None,
        }
    }
}

impl ChunkyIterator<'_> {
    pub fn from_bytes(bytes: &[u8]) -> ChunkyIterator {
        ChunkyIterator {
            bytes,
            extra_bytes: Vec::with_capacity(64),
        }
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
