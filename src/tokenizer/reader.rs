use core::slice::memchr::memchr;

pub struct StrReader<'a> {
    pub slice: &'a str,
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self { slice, pos: 0, col: 0 }
    }
}

pub(crate) trait Reader {
    fn peek_byte(&mut self) -> Option<u8>;
    fn peek_byte_is(&mut self, needle: u8) -> bool;
    fn consume_bytes(&mut self, amount: usize);
    fn slice_bytes(&self, start: usize, end: usize) -> &[u8];

    fn try_read_slice(&mut self, needle: &str, case_sensitive: bool) -> bool;
    #[inline(always)]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        self.try_read_slice(needle, true)
    }
    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead;
    fn skip_space_tab(&mut self) -> usize;
    fn read_line(&mut self) -> (usize, usize);
}

impl<'r> Reader for StrReader<'r> {
    fn peek_byte(&mut self) -> Option<u8> {
        match self.slice.as_bytes().get(self.pos) {
            Some(x) => Some(*x),
            _ => None,
        }
    }

    fn peek_byte_is(&mut self, needle: u8) -> bool {
        match self.slice.as_bytes().get(self.pos) {
            Some(x) if x == &needle => true,
            _ => false,
        }
    }

    fn consume_bytes(&mut self, amount: usize) {
        self.pos += amount;
    }

    fn slice_bytes(&self, start: usize, end: usize) -> &'r [u8] {
        &self.slice.as_bytes()[start..end]
    }

    fn try_read_slice(&mut self, needle: &str, case_sensitive: bool) -> bool {
        if self.slice.len() < needle.len() {
            return false;
        }

        let read = if case_sensitive {
            self.slice.as_bytes()[self.pos..self.pos + needle.len()].starts_with(needle.as_bytes())
        } else {
            needle.as_bytes().iter().enumerate().all(|(offset, char)| {
                self.slice.as_bytes()[self.pos + offset].to_ascii_lowercase()
                    == char.to_ascii_lowercase()
            })
        };

        if read {
            self.pos += needle.len();
        }
        read
    }

    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead {
        let (read, n) = match fast_find(needle, &self.slice.as_bytes()[self.pos..]) {
            Some(0) => (FastRead::Char(self.slice.as_bytes()[self.pos]), 1),
            Some(size) => (FastRead::InterNeedle(self.pos, self.pos + size), size),
            None => (FastRead::EOF, 0),
        };
        self.pos += n;
        read
    }

    fn skip_space_tab(&mut self) -> usize {
        let n = self.slice.as_bytes()[self.pos..]
            .iter()
            .position(|b| !is_tab_space(*b))
            .unwrap_or(0);
        self.consume_bytes(n);
        n
    }

    fn read_line(&mut self) -> (usize, usize) {
        if let Some(n) = fast_find(&[b'\r', b'\n'], &self.slice.as_bytes()[self.pos..]) {
            let x = (self.pos, self.pos+n);
            self.consume_bytes(n+1);
            if self.peek_byte_is(b'\n') {
                self.consume_bytes(1);
            };
            self.col = 0;
            return x;
        }
        (0, 0)
    }
}

#[test]
pub fn test_readline() {
    let mut win_reader = StrReader::new("#   |\r\n");
    let mut lin_reader = StrReader::new("#   |\n");
    let mut mac_reader = StrReader::new("#   |\r");

    assert_eq!((0, 5), win_reader.read_line());
    assert_eq!(None, win_reader.peek_byte());
    assert_eq!((0, 5), lin_reader.read_line());
    assert_eq!(None, lin_reader.peek_byte());
    assert_eq!((0, 5), mac_reader.read_line());
    assert_eq!(None, mac_reader.peek_byte());
}

#[inline]
pub(crate) fn is_tab_space(b: u8) -> bool {
    match b {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn fast_find(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    #[cfg(feature = "jetscii")]
    {
        debug_assert!(needle.len() <= 16);
        let mut needle_arr = [0; 16];
        needle_arr[..needle.len()].copy_from_slice(needle);
        jetscii::Bytes::new(needle_arr, needle.len() as i32, |b| needle.contains(&b)).find(haystack)
    }

    #[cfg(not(feature = "jetscii"))]
    {
        haystack.iter().position(|b| needle.contains(b))
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FastRead {
    Char(u8),
    InterNeedle(usize, usize),
    EOF,
}
