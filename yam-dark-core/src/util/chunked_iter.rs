#![allow(unused)]

const CHUNK_SIZE: usize = 64;
const EMPTY_CHUNK: [u8; CHUNK_SIZE] = [b' '; CHUNK_SIZE];

pub struct ChunkArrayIter<'a> {
    v: &'a [u8],
    rem: &'a [u8],
}

impl<'a> ChunkArrayIter<'a> {
    #[inline]
    pub fn from_bytes(slice: &'a [u8]) -> Self {
        let rem = slice.len() % CHUNK_SIZE;
        let fst_len = slice.len() - rem;
        // SAFETY: 0 <= fst_len <= slice.len() by construction above
        let (fst, snd) = unsafe { slice.split_at_unchecked(fst_len) };
        Self { v: fst, rem: snd }
    }

    #[must_use]
    pub fn remainder(&self) -> &'a [u8] {
        self.rem
    }
}

impl<'a> Iterator for ChunkArrayIter<'a> {
    type Item = &'a [u8; 64];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8; 64]> {
        if self.v.len() < CHUNK_SIZE {
            None
        } else {
            let (fst, snd) = self.v.split_at(CHUNK_SIZE);
            self.v = snd;
            Some(unsafe { &*fst.as_ptr().cast::<[u8; 64]>() })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.v.len() / CHUNK_SIZE;
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (start, overflow) = n.overflowing_mul(CHUNK_SIZE);
        if start >= self.v.len() || overflow {
            self.v = &[];
            None
        } else {
            let (_, snd) = self.v.split_at(start);
            self.v = snd;
            self.next()
        }
    }
}

impl ExactSizeIterator for ChunkArrayIter<'_> {}

#[test]
fn test_chunk() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let x = [a, b].concat();
    let mut iter = ChunkArrayIter::from_bytes(&x);
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
    let mut iter = ChunkArrayIter::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), Some(&b));

    let mut c = [b' '; 64];
    c[0] = 3;
    assert_eq!(iter.next(), None);
}

#[test]
fn test_chunk_rem_minus() {
    let a = [0u8; 64];
    let b = [1u8; 64];
    let mut x = [a, b].concat();
    x.drain(67..);
    let mut iter = ChunkArrayIter::from_bytes(&x);
    assert_eq!(iter.next(), Some(&a));

    let mut c = [b' '; 64];
    c[0] = 1;
    c[1] = 1;
    c[2] = 1;
    assert_eq!(iter.next(), None);
}
