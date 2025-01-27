const CHUNK_SIZE : usize = 64;

pub(crate) struct ChunkyIterator<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for ChunkyIterator<'a> {
    type Item = &'a [u8; CHUNK_SIZE];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((chunk, rest)) = self.bytes.split_first_chunk::<CHUNK_SIZE>() {
            self.bytes = rest;
            return Some(chunk);
        }
        None
    }
}

impl<'a> ChunkyIterator<'a> {
    
    pub(crate) fn from_bytes(bytes: &[u8]) -> ChunkyIterator {
        ChunkyIterator {
            bytes
        }
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
    let mut iter = ChunkyIterator {
        bytes: &x,
    };
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
    let mut iter = ChunkyIterator {
        bytes: &x,
    };
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
    let mut iter = ChunkyIterator {
        bytes: &x,
    };
    assert_eq!(iter.next(), Some(&a));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.finalize(), &[1, 1, 1]);
}