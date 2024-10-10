#![allow(unused)]
use core::slice::from_raw_parts;

pub struct ChunkyIterator<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for ChunkyIterator<'a> {
    type Item = &'a [u8; 64];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.len() < 64 {
            None
        } else {
            let len = self.bytes.len();
            let ptr = self.bytes.as_ptr();
            // SAFETY: We manually verified the bounds of the split.
            let (first, tail) = unsafe {
                (
                    from_raw_parts(ptr, 64),
                    from_raw_parts(ptr.add(64), len - 64),
                )
            };
            self.bytes = tail;
            // SAFETY: We explicitly check for the correct number of elements,
            //   and do not let the references outlive the slice.
            Some(unsafe { &*(first.as_ptr() as *const [u8; 64]) })
        }
    }
}

impl<'a> ChunkyIterator<'a> {
    pub fn from_bytes(bytes: &[u8]) -> ChunkyIterator {
        ChunkyIterator { bytes }
    }
    pub(crate) fn finalize(&self) -> &[u8] {
        self.bytes
    }
}

#[test]
fn test_chunk() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let x = [a, b].concat();
    let mut iter = ChunkyIterator { bytes: &x };
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), Some(&b));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.finalize(), &[]);
}

#[test]
fn test_chunk_rem() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let mut x = [a, b].concat();
    x.push(3);
    let mut iter = ChunkyIterator { bytes: &x };
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), Some(&b));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.finalize(), &[3]);
}

#[test]
fn test_chunk_rem_minus() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let mut x = [a, b].concat();
    x.drain(67..);
    let mut iter = ChunkyIterator { bytes: &x };
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.finalize(), &[1, 1, 1]);
}
