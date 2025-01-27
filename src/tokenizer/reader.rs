use memchr::memchr2;

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
    fn pos(&self) -> usize;
    fn col(&self) -> usize;
    fn peek_byte(&mut self) -> Option<u8>;
    fn peek_byte_is(&mut self, needle: u8) -> bool;
    fn consume_bytes(&mut self, amount: usize);
    fn slice_bytes(&self, start: usize, end: usize) -> &[u8];

    #[inline(always)]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn find_next_non_whitespace(&self) -> Option<usize>;
    fn find_fast2_offset(&self, needle1: u8, needle2: u8) -> Option<(usize, usize)>;
    fn skip_space_tab(&mut self) -> usize;
    fn read_line(&mut self) -> (usize, usize);
    fn read_non_comment_line(&mut self) -> (usize, usize);
}

impl<'r> Reader for StrReader<'r> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn col(&self) -> usize {
        self.col
    }

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

    #[inline(always)]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        if self.slice.as_bytes()[self.pos..self.pos + needle.len()].starts_with(needle.as_bytes()) {
            self.pos += needle.len();
            return true;
        }
        false
    }

    fn find_next_non_whitespace(&self) -> Option<usize> {
        self.slice.as_bytes()[self.pos..]
            .iter()
            .position(|p| !is_whitespace(*p))
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
        let mut iter = memchr::memchr3_iter(b'\r', b'\n', b'#', content);
        let mut end = self.pos;
        let mut consume = 0usize;

        if let Some((new_end, c)) = iter.next().map(|p| (p, content[p])) {
            end = new_end;
            consume = end + 1;

            if c == b'\n' {
                self.consume_bytes(consume);
                self.col = 0;
                return (start, end);
            }
        }
        while let Some(pos) = iter.next() {
            let ascii = content[pos];
            if ascii == b'\r' && pos < content.len() - 1 && content[pos + 1] == b'\n' {
                self.consume_bytes(pos + 2);
                self.col = 0;
                return (start, end);
            } else if ascii == b'\r' || ascii == b'\n' {
                self.consume_bytes(pos + 1);
                self.col = 0;
                return (start, end);
            }
        }

        (start, end)
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

#[test]
pub fn test_read2lines() {
    let mut win_reader = StrReader::new("#   |\r\n \r\n");
    let mut lin_reader = StrReader::new("#   |\n\n");
    let mut mac_reader = StrReader::new("#   |\r\r");

    assert_eq!((0, 5), win_reader.read_line());
    assert_eq!(Some(b' '), win_reader.peek_byte());
    assert_eq!(0, win_reader.col);
    assert_eq!((7, 8), win_reader.read_line());
    assert_eq!(0, win_reader.col);
    assert_eq!(None, win_reader.peek_byte());

    assert_eq!((0, 5), lin_reader.read_line());
    assert_eq!(Some(b'\n'), lin_reader.peek_byte());
    assert_eq!(0, lin_reader.col);
    assert_eq!((6, 6), lin_reader.read_line());
    assert_eq!(0, lin_reader.col);
    assert_eq!(None, lin_reader.peek_byte());

    assert_eq!((0, 5), mac_reader.read_line());
    assert_eq!(Some(b'\r'), mac_reader.peek_byte());
    assert_eq!(0, mac_reader.col);
    assert_eq!((6, 6), mac_reader.read_line());
    assert_eq!(0, mac_reader.col);
    assert_eq!(None, mac_reader.peek_byte());
}

#[test]
pub fn read_non_comment_line() {
    let mut win_reader = StrReader::new("   # # \r\n");
    let mut mac_reader = StrReader::new("   # # \r");
    let mut lin_reader = StrReader::new("   # # \n");

    assert_eq!((0, 3), win_reader.read_non_comment_line());
    assert_eq!(None, win_reader.peek_byte());
    assert_eq!(9, win_reader.pos);
    assert_eq!(0, win_reader.col);

    assert_eq!((0, 3), mac_reader.read_non_comment_line());
    assert_eq!(None, mac_reader.peek_byte());
    assert_eq!(8, mac_reader.pos);
    assert_eq!(0, mac_reader.col);

    assert_eq!((0, 3), lin_reader.read_non_comment_line());
    assert_eq!(None, lin_reader.peek_byte());
    assert_eq!(8, lin_reader.pos);
    assert_eq!(0, lin_reader.col);
}

#[inline]
pub(crate) fn is_tab_space(b: u8) -> bool {
    match b {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn is_whitespace(b: u8) -> bool {
    match b {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FastRead {
    Char(u8),
    InterNeedle(usize, usize),
    EOF,
}
