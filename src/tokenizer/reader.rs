use memchr::{memchr, memchr2, memchr3};

pub struct StrReader<'a> {
    pub slice: &'a str,
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self {
            slice,
            pos: 0,
            col: 0,
        }
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
    fn find_fast2_offset(&self, needle1: u8, needle2: u8) -> Option<(usize, usize)>;
    fn skip_space_tab(&mut self) -> usize;
    fn read_line(&mut self) -> (usize, usize);
    fn read_non_comment_line(&mut self) -> (usize, usize);
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

    #[inline(always)]
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

    fn find_fast2_offset(&self, needle1: u8, needle2: u8) -> Option<(usize, usize)> {
        if let Some(n) = memchr2(needle1, needle2, &self.slice.as_bytes()[self.pos..]) {
            return Some((self.pos, self.pos + n));
        }
        None
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
        let start = self.pos;
        let content = &self.slice.as_bytes()[start..];
        let (n, consume) = memchr::memchr2_iter(b'\r', b'\n', content)
            .next()
            .map_or((0, 0), |p| {
                if content[p] == b'\r' && p < content.len() - 1 && content[p + 1] == b'\n' {
                    (p, p + 2)
                } else {
                    (p, p + 1)
                }
            });
        self.consume_bytes(consume);
        self.col = 0;
        (start, start + n)
    }

    fn read_non_comment_line(&mut self) -> (usize, usize) {
        let start = self.pos;
        let content = &self.slice.as_bytes()[start..];
        let consume: usize = memchr::memchr3_iter(b'\r', b'\n', b'#', content)
            .map(|p| {
                if content[p] == b'\r' && content[p + 1] == b'\n' {
                    p
                } else {
                    p - 1
                }
            })
            .sum();
        self.consume_bytes(consume);
        if content[start + consume] == b'\r' || content[start + consume] == b'\n' {
            self.col = 0;
        }
        (start, start + consume)
    }
}

#[test]
pub fn test_readline() {
    let mut win_reader = StrReader::new("#   |\r\n");
    let mut lin_reader = StrReader::new("#   |\n");
    let mut mac_reader = StrReader::new("#   |\r");

    assert_eq!((0, 5), win_reader.read_line());
    assert_eq!(None, win_reader.peek_byte());
    assert_eq!(0, win_reader.col);

    assert_eq!((0, 5), lin_reader.read_line());
    assert_eq!(None, lin_reader.peek_byte());
    assert_eq!(0, lin_reader.col);

    assert_eq!((0, 5), mac_reader.read_line());
    assert_eq!(None, mac_reader.peek_byte());
    assert_eq!(0, mac_reader.col);
}

#[inline]
pub(crate) fn is_tab_space(b: u8) -> bool {
    match b {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FastRead {
    Char(u8),
    InterNeedle(usize, usize),
    EOF,
}
