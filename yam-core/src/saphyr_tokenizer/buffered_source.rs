use crate::Source;
use crate::saphyr_tokenizer::char_utils::is_break;
use crate::saphyr_tokenizer::scanner::SkipTabs;
use crate::util::u8x64_eq;
use alloc::vec::Vec;
use core::iter::Copied;
use core::mem::{MaybeUninit, transmute};
use core::slice;
use core::slice::Iter;

const MAX_LEN: usize = 64;

#[allow(dead_code)]
pub struct BufferedBytesSource<T> {
    input: T,
    buf: [MaybeUninit<u8>; MAX_LEN],
    len: usize,
}

impl<T: Iterator<Item = u8>> BufferedBytesSource<T> {
    pub fn new(input: T) -> Self {
        let mut x = Self {
            input,
            buf: [MaybeUninit::uninit(); MAX_LEN],
            len: 0,
        };
        x.fill_buf_to_max();
        x
    }

    fn fill_buf_to_max(&mut self) {
        for x in self.len..self.buf_max_len() {
            match self.input.next() {
                Some(byt) => {
                    self.buf[x].write(byt);
                    self.len += 1;
                }
                None => break,
            }
        }
    }

    pub fn get_max_buf(&self) -> Option<[u8; 64]> {
        if self.len < self.buf_max_len() {
            return None;
        }
        let buf = unsafe { transmute::<[MaybeUninit<u8>; 64], [u8; 64]>(self.buf) };
        Some(buf)
    }

    pub fn get_buf(&self) -> &[u8] {
        unsafe {
            // SAFETY: This is safe because self.len guarantees that the buf is initialized
            // up to self.len position
            slice::from_raw_parts(self.buf.as_ptr().cast(), self.len)
        }
    }
}

impl<'a> BufferedBytesSource<Copied<Iter<'a, u8>>> {
    pub fn from_bstr(input: &'a [u8]) -> Self {
        let mut x = Self {
            input: input.iter().copied(),
            buf: [MaybeUninit::uninit(); MAX_LEN],
            len: 0,
        };
        x.fill_buf_to_max();
        x
    }
}
impl<T: Iterator<Item = u8>> Source for BufferedBytesSource<T> {
    fn peek_arbitrary(&self, n: usize) -> u8 {
        debug_assert!(n < self.buf_max_len());
        if n >= self.len {
            return b'\0';
        }
        unsafe { self.buf[n].assume_init() }
    }

    fn peek(&self) -> u8 {
        self.peek_arbitrary(0)
    }

    fn peek_char(&self) -> char {
        // SAFETY: This takes up to 4 bytes. `4.min(self.len)` guarantees it won't see any
        // uninit values
        let slice = unsafe { slice::from_raw_parts(self.buf.as_ptr().cast(), 4.min(self.len)) };
        // SAFETY: Converts up to the next 4 bytes (currently UTF-8) maximum
        // to a char using from_utf8_unchecked. It should contain at least one valid char, which is
        // all we care about.
        let mut bytes = unsafe { str::from_utf8_unchecked(slice).chars() };
        bytes.next().unwrap()
    }

    fn skip(&mut self, n: usize) {
        let consume = n.min(self.len);
        let skip = n.saturating_sub(self.buf_max_len());

        self.input.nth(skip);

        self.buf.copy_within(consume..self.len, 0);
        self.len = self.len.saturating_sub(consume);

        self.fill_buf_to_max();
    }

    fn buf_max_len(&self) -> usize {
        MAX_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.len == 0
    }

    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> (u32, Result<SkipTabs, &'static str>) {
        todo!()
    }

    fn next_is_z(&self) -> bool {
        self.buf_is_empty()
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>) {
        let mut pos = None;
        while let Some(x) = self.get_max_buf() {
            // bitmask for \r or \n
            let break_bitmask = u8x64_eq(&x, b'\n') | u8x64_eq(&x, b'\r');
            let first_nl = break_bitmask.trailing_zeros() as usize;

            if break_bitmask != 0 {
                pos = Some(first_nl);
                break;
            }

            vec.extend_from_slice(&x[..]);
            self.skip(64)
        }
        let buf = self.get_buf();

        // we get the value from while loop above, or we search remaining buf
        let found_pos = pos.unwrap_or(buf.iter().position(|&c| is_break(c)).unwrap_or(0));
        vec.extend_from_slice(&buf[..found_pos]);
        self.skip(found_pos);
    }
}

#[cfg(test)]
mod tests {
    use crate::Source;
    use crate::saphyr_tokenizer::buffered_source::BufferedBytesSource;

    #[test]
    fn test_create() {
        let source = BufferedBytesSource::from_bstr(b"Hello, world!");
        assert_eq!(source.len, 13);
        assert_eq!(source.peek(), b'H');
        assert_eq!(source.peek_n1(), b'e');
    }

    #[test]
    fn test_skip() {
        let mut source = BufferedBytesSource::from_bstr(b"Hello, world!");
        assert_eq!(source.len, 13);
        assert_eq!(source.peek(), b'H');
        assert_eq!(source.peek_n1(), b'e');

        source.skip(3);
        assert_eq!(source.peek(), b'l');
        assert_eq!(source.peek_n1(), b'o');

        source.skip(4);
        assert_eq!(source.peek(), b'w');
        assert_eq!(source.peek_n1(), b'o');
    }

    #[test]
    fn test_skip_big() {
        let mut source = BufferedBytesSource::from_bstr(
            br#"Lorem ipsum dolor sit amet, 
            consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet, 
            ornare vitae erat. Aenean bibendum arcu et risus auctor, 
            nec finibus arcu finibus. Integer ut congue metus, non hendrerit nunc. 
            Ut ornare efficitur nisl, sed ullamcorper risus feugiat at. 
            Aenean ut mi a nulla pellentesque aliquet quis vitae lorem. Vestibulum semper elit"#,
        );
        assert_eq!(source.len, 64);
        assert_eq!(source.peek(), b'L');
        assert_eq!(source.peek_n1(), b'o');

        // Skip characters to ornare vitae
        source.skip(130);
        assert_eq!(source.peek(), b'o');
        assert_eq!(source.peek_n1(), b'r');
        assert_eq!(source.peek_n2(), b'n');
    }

    #[test]
    fn test_create_empty() {
        let mut source = BufferedBytesSource::from_bstr(b"");
        assert_eq!(source.len, 0);
        assert_eq!(source.peek(), b'\0');
        assert_eq!(source.peek_n1(), b'\0');

        source.skip(130);
        assert_eq!(source.peek(), b'\0');
        assert_eq!(source.peek_n1(), b'\0');
    }
}
