use crate::parsing::Source;
use crate::parsing::char_utils::is_break;
use crate::parsing::scanner::SkipTabs;
use crate::parsing::source::shared_skip_ws_to_eol;
use crate::util::{BitOps, U8X32};
use alloc::vec::Vec;
use core::iter::Copied;
use core::mem::{MaybeUninit, transmute};
use core::slice;
use core::slice::Iter;

pub(crate) const MAX_LEN: usize = 32;

#[allow(dead_code)]
pub struct BufferedBytesSource<T> {
    /// The input source. Usually a byte iterator
    input: T,
    /// The buffer of `MaybeUninit` bytes
    chunk_buf: [MaybeUninit<u8>; MAX_LEN],
    /// How much of buffer has been filled
    filled: usize,
}

pub type BufferedBytesSourceIter<'a> = BufferedBytesSource<Copied<Iter<'a, u8>>>;

impl<T: Iterator<Item = u8>> BufferedBytesSource<T> {
    fn fill_buf_to_max(&mut self) {
        for x in self.filled..self.buf_max_len() {
            match self.input.next() {
                Some(byt) => {
                    // SAFETY: This will always be in buffer len
                    unsafe {
                        self.chunk_buf.get_unchecked_mut(x).write(byt);
                    }
                    self.filled += 1;
                }
                None => break,
            }
        }
    }

    pub fn get_max_buf(&self) -> Option<[u8; MAX_LEN]> {
        if self.filled < self.buf_max_len() {
            return None;
        }
        let buf = unsafe { transmute::<[MaybeUninit<u8>; MAX_LEN], [u8; MAX_LEN]>(self.chunk_buf) };
        Some(buf)
    }

    pub fn get_buf(&self) -> &[u8] {
        unsafe {
            // SAFETY: This is safe because self.len guarantees that the buf is initialized
            // up to self.len position
            slice::from_raw_parts(self.chunk_buf.as_ptr().cast(), self.filled)
        }
    }
}

#[allow(dead_code)]
impl<'a> BufferedBytesSource<Copied<Iter<'a, u8>>> {
    #[must_use]
    pub fn new_from_bytes(input: &'a [u8]) -> Self {
        let mut x = Self {
            input: input.iter().copied(),
            chunk_buf: [MaybeUninit::uninit(); MAX_LEN],
            filled: 0,
        };
        x.fill_buf_to_max();
        x
    }

    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self::new_from_bytes(input.as_bytes())
    }
}
unsafe impl<T: Iterator<Item = u8>> Source for BufferedBytesSource<T> {
    unsafe fn peek_unsafe(&self, n: usize) -> u8 {
        unsafe { self.chunk_buf[n].assume_init() }
    }

    fn peek_checked(&self, n: usize) -> Option<u8> {
        if n >= self.filled || n >= self.buf_max_len() {
            return None;
        }
        unsafe { Some(self.peek_unsafe(n)) }
    }

    fn peek_char(&self) -> char {
        // SAFETY: This takes up to 4 bytes. `4.min(self.len)` guarantees it won't see any
        // uninit values
        let slice =
            unsafe { slice::from_raw_parts(self.chunk_buf.as_ptr().cast(), 4.min(self.filled)) };
        // SAFETY: Converts up to the next 4 bytes (currently UTF-8) maximum
        // to a char using from_utf8_unchecked. It should contain at least one valid char, which is
        // all we care about.
        let mut bytes = unsafe { str::from_utf8_unchecked(slice).chars() };
        bytes.next().unwrap()
    }

    fn buf_max_len(&self) -> usize {
        MAX_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.filled == 0
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>) {
        let mut pos = None;
        while let Some(x) = self.get_max_buf() {
            // bitmask for \r or \n
            let fake_simd = U8X32::from_array(x);
            let break_bitmask = fake_simd.comp_to_bitmask(b'\r') | fake_simd.comp_to_bitmask(b'\n');
            let first_nl = break_bitmask.trailing_zeros() as usize;

            if break_bitmask != 0 {
                pos = Some(first_nl);
                break;
            }

            vec.extend_from_slice(&x[..]);
            self.skip(MAX_LEN);
        }
        let buf = self.get_buf();

        // we get the value from while loop above, or we search remaining buf
        let found_pos =
            pos.unwrap_or_else(|| buf.iter().position(|&c| is_break(c)).unwrap_or(buf.len()));
        vec.extend_from_slice(&buf[..found_pos]);
        self.skip(found_pos);
    }

    fn skip(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let consume = n.min(self.filled);
        let skip = n.saturating_sub(self.buf_max_len() + 1);

        if skip > 0 {
            self.input.nth(skip);
        }

        self.chunk_buf.copy_within(consume..self.filled, 0);

        let new_len = self.filled.saturating_sub(consume);
        self.filled = new_len;
        self.fill_buf_to_max();
    }

    fn skip_spaces(&mut self, max: usize) -> usize {
        let mut n_chars = 0;

        while let Some(x) = self.get_max_buf() {
            let fake_simd = U8X32::from_array(x);
            let spaces = fake_simd.comp_to_bitmask(b' ');
            let n = (spaces.trailing_ones() as usize).min(max);

            self.skip(n);
            n_chars += n;
            if n < 32 || n_chars >= max {
                return n_chars;
            }
        }
        let debug = self.get_buf();
        let slow_path = debug
            .iter()
            .position(|&c| c != b' ')
            .unwrap_or(debug.len())
            .min(max);

        self.skip(slow_path);
        n_chars += slow_path;
        n_chars
    }

    #[allow(clippy::cast_possible_truncation)]
    fn skip_ws_to_eol(
        &mut self,
        skip_tab: bool,
        mut has_yaml_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>) {
        let mut any_tabs = false;
        let mut consumed_bytes = 0u32;
        let tabs_mul = if skip_tab { 1 } else { 0 };

        // Skip whole buffers of whitespace at once. Anything else (comments, errors, the tail of
        // the input) is left to `shared_skip_ws_to_eol`.
        while let Some(x) = self.get_max_buf() {
            let fake_simd = U8X32::from_array(x);
            let spaces = fake_simd.comp_to_bitmask(b' ');
            let tabs = fake_simd.comp_to_bitmask(b'\t') * tabs_mul;
            let n = (spaces | tabs).trailing_ones();
            let prefix = if n == 32 { u32::MAX } else { (1 << n) - 1 };
            has_yaml_ws |= spaces & prefix != 0;
            any_tabs |= tabs & prefix != 0;

            self.skip(n as usize);
            consumed_bytes += n;
            if n < 32 {
                break;
            }
        }

        shared_skip_ws_to_eol(self, skip_tab, consumed_bytes, any_tabs, has_yaml_ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::Source;
    use crate::parsing::buffered_source::{BufferedBytesSource, MAX_LEN};
    use crate::parsing::scanner::SkipTabs;
    use alloc::vec::Vec;

    #[test]
    fn test_skip_ws_minimal() {
        let mut source = BufferedBytesSource::new("                       ,     a a");

        assert_eq!(source.peekz(0), b' ');
        assert_eq!(source.skip_spaces(0), 0);
        assert_eq!(source.peekz(0), b' ');
        assert_eq!(source.skip_spaces(5), 5);
        assert_eq!(source.peekz(0), b' ');
    }

    #[test]
    fn test_create() {
        let source = BufferedBytesSource::new_from_bytes(b"Hello, world!");
        assert_eq!(source.filled, 13);
        assert_eq!(source.peekz(0), b'H');
        assert_eq!(source.peekz(1), b'e');
    }

    #[test]
    fn test_skip() {
        let mut source = BufferedBytesSource::new_from_bytes(b"Hello, world!");
        assert_eq!(source.filled, 13);
        assert_eq!(source.peekz(0), b'H');
        assert_eq!(source.peekz(1), b'e');

        source.skip(3);
        assert_eq!(source.peekz(0), b'l');
        assert_eq!(source.peekz(1), b'o');

        source.skip(4);
        assert_eq!(source.peekz(0), b'w');
        assert_eq!(source.peekz(1), b'o');
    }

    #[test]
    fn test_skip_big() {
        let mut source = BufferedBytesSource::new_from_bytes(
            br#"Lorem ipsum dolor sit amet, 
            consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet, 
            ornare vitae erat. Aenean bibendum arcu et risus auctor, 
            nec finibus arcu finibus. Integer ut congue metus, non hendrerit nunc. 
            Ut ornare efficitur nisl, sed ullamcorper risus feugiat at. 
            Aenean ut mi a nulla pellentesque aliquet quis vitae lorem. Vestibulum semper elit"#,
        );
        assert_eq!(source.filled, MAX_LEN);
        assert_eq!(source.peekz(0), b'L');
        assert_eq!(source.peekz(1), b'o');

        // Skip characters to ornare vitae
        source.skip(131);
        assert_eq!(source.peekz(0), b'o');
        assert_eq!(source.peekz(1), b'r');
        assert_eq!(source.peekz(2), b'n');
    }

    #[test]
    fn test_create_empty() {
        let mut source = BufferedBytesSource::new_from_bytes(b"");
        assert_eq!(source.filled, 0);
        assert_eq!(source.peekz(0), b'\0');
        assert_eq!(source.peekz(1), b'\0');

        source.skip(130);
        assert_eq!(source.peekz(0), b'\0');
        assert_eq!(source.peekz(1), b'\0');
    }

    #[test]
    fn test_push_breakz() {
        let mut source = BufferedBytesSource::new_from_bytes(b"Lorem ipsum dolor sit amet,
                                                        consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet,
                                                        ornare vitae erat. Aenean bibendum arcu et risus auctor,");
        let mut dest = Vec::new();
        source.push_non_breakz_chr(&mut dest);
        assert_eq!(source.peekz(0), b'\n');
        assert_eq!(
            str::from_utf8(&dest).unwrap(),
            "Lorem ipsum dolor sit amet,"
        );

        // Skip newline
        source.skip(1);

        dest.clear();
        source.push_non_breakz_chr(&mut dest);
        assert_eq!(source.peekz(0), b'\n');
        assert_eq!(
            str::from_utf8(&dest).unwrap(),
            "                                                        consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet,"
        );
    }

    const YAML_WS_TABS: &str = "                                                                        \t\t         \
                    test";

    #[test]
    fn test_tabs() {
        let mut source = BufferedBytesSource::new(YAML_WS_TABS);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert!(res1.1.is_ok());
        assert_eq!(res1.0, 83);

        let skip_tabs = res1.1.unwrap();
        assert!(skip_tabs.found_tabs());
        assert!(skip_tabs.has_valid_yaml_ws());

        let mut source = BufferedBytesSource::new(YAML_WS_TABS);
        assert_eq!(source.peekz(0), b' ');
        let res2 = source.skip_ws_to_eol(false, false);
        assert!(res2.1.is_ok());
        assert_eq!(res2.0, 72);
    }

    #[test]
    fn skip_ws_doesnt_skip_non_ws() {
        let folded_ex = "  - folded example where \nyou need to skip only to the `-` character\n not to this part";
        let mut source = BufferedBytesSource::new(folded_ex);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert_eq!(res1.0, 2);
        assert_eq!(
            res1.1,
            Ok(SkipTabs::Result {
                any_tabs: false,
                has_yaml_ws: true
            })
        );
    }

    #[test]
    #[cfg(not(feature = "comment"))]
    fn skip_comment() {
        let folded_ex =
            "   # commented out \nyou need to skip only to the `-` character\n not to this part";
        let mut source = BufferedBytesSource::new(folded_ex);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert_eq!(res1.0, 19);
        assert_eq!(
            res1.1,
            Ok(SkipTabs::Result {
                any_tabs: false,
                has_yaml_ws: true
            })
        );
        assert_eq!(source.peekz(0), b'\n');
    }

    #[test]
    #[cfg(not(feature = "comment"))]
    fn skip_comments_multiline() {
        let multiline_comment = r#"
%YAML 1.3 # Attempt parsing
          # with a warning
---"#;
        let mut source = BufferedBytesSource::new(multiline_comment);
        assert_eq!(source.peekz(0), b'\n');
        source.skip(10);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert_eq!(res1.0, 18);
        assert_eq!(
            res1.1,
            Ok(SkipTabs::Result {
                any_tabs: false,
                has_yaml_ws: true
            })
        );
        assert_eq!(source.peekz(0), b'\n');
    }

    #[test]
    #[cfg(not(feature = "comment"))]
    fn skip_comments_multiline2() {
        let multiline_comment = r#"
 - test minor            
                                           
---"#;
        let mut source = BufferedBytesSource::new(multiline_comment);
        assert_eq!(source.peekz(0), b'\n');
        let res1 = source.skip_ws_to_eol(true, false);
        assert_eq!(res1.0, 0);
        assert_eq!(
            res1.1,
            Ok(SkipTabs::Result {
                any_tabs: false,
                has_yaml_ws: false
            })
        );
        assert_eq!(source.peekz(0), b'\n');
    }

    #[test]
    fn skip_whitespace() {
        let multiline_comment =
            r#" { }                                                           "#;
        let mut source = BufferedBytesSource::new(multiline_comment);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert_eq!(res1.0, 1);
        assert_eq!(
            res1.1,
            Ok(SkipTabs::Result {
                any_tabs: false,
                has_yaml_ws: true
            })
        );
    }

    #[test]
    fn test_empty() {
        let x = "";
        let source = BufferedBytesSource::new(x);
        assert_eq!(source.peekz(32), b'\0');
    }
}
