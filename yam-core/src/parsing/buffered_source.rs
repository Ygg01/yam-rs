use crate::parsing::{Source, char_utils::is_blank_or_breakz};
use crate::parsing::char_utils::is_break;
use crate::parsing::scanner::SkipTabs;
use crate::parsing::source::shared_skip_ws_to_eol;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::iter::Copied;
use core::mem::MaybeUninit;
use core::slice;
use core::slice::Iter;
use fearless_simd::{Level, Simd, SimdBase, SimdMask, dispatch};

pub(crate) const BUF_MAX_LEN: usize = 128;

#[allow(dead_code)]
pub struct BufferedBytesSource<T> {
    /// The input source. Usually a byte iterator
    input: T,
    /// The buffer of `MaybeUninit` bytes
    chunk_buf: Box<[MaybeUninit<u8>; BUF_MAX_LEN]>,
    /// How much of the buffer has been filled.
    filled: usize,
    /// Position of the head in the buffer.
    pos: usize,
}

pub type BufferedBytesSourceIter<'a> = BufferedBytesSource<Copied<Iter<'a, u8>>>;

impl<T: Iterator<Item = u8>> BufferedBytesSource<T> {
    #[inline]
    /// Replaces entire buffer with new data from iterator.
    fn fill_buf_to_max(&mut self) {
        self.fill_buf_from(0);
    }

    fn fill_buf_from(&mut self, pos: usize) {
        // SAFETY: pos can't be over BUF_MAX_LEN
        assert!(pos <= BUF_MAX_LEN);
        // SAFETY: filled starts from pos and overwrites until BUF_MAX_LEN
        self.filled = pos;
        for x in pos..self.buf_max_len() {
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

    fn reset_head_to_start(&mut self) {
        if self.pos == 0 {
            return;
        }
        let n = self.filled.saturating_sub(self.pos);
        self.chunk_buf.copy_within(self.pos..self.filled, 0);
        self.fill_buf_from(n);
        self.pos = 0;
    }

    pub fn get_chunk_buf(&self) -> Option<&[u8]> {
        if self.filled < self.buf_max_len() {
            return None;
        }
        let buf = unsafe {
            // SAFETY: This is safe because self.len guarantees that the buf is initialized
            // up to self.len position
            slice::from_raw_parts(self.chunk_buf.as_ptr().cast(), self.filled)
        };
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
            chunk_buf: Box::new([MaybeUninit::uninit(); BUF_MAX_LEN]),
            pos: 0,
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
        unsafe { self.chunk_buf.get_unchecked(self.pos + n).assume_init() }
    }

    fn peek_checked(&self, n: usize) -> Option<u8> {
        if (self.pos + n) >= self.filled {
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
        BUF_MAX_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.filled < BUF_MAX_LEN && self.pos >= self.filled
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>) {
        let level = Level::new();
        self.reset_head_to_start();
        dispatch!(level, simd => push_non_break_simd(simd, self, vec));
    }

    fn skip(&mut self, n: usize) {
        // No skip, so no-op necessary
        if n == 0 {
            return;
        }

        // Within buffer so just update head pos
        if n + self.pos <= self.filled {
            self.pos += n;
            return;
        }

        // Stuff is outside of buffer. Move head to 0 and repopulate the buffer 
        let skip = (n + self.pos).saturating_sub(self.buf_max_len() + 1);
        if skip > 0 {
            self.input.nth(skip);
        }    
        
        self.pos = 0;
        self.fill_buf_to_max();
    }

    fn skip_spaces(&mut self, max: usize) -> usize {
        let level = Level::new();
        self.reset_head_to_start();
        dispatch!(level, simd => skip_spaces_simd(simd, self, max))
    }

    #[allow(clippy::cast_possible_truncation)]
    fn skip_ws_to_eol(
        &mut self,
        skip_tab: bool,
        mut has_yaml_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>) {
        let level = Level::new();
        self.reset_head_to_start();
        dispatch!(level, simd => skip_ws_to_eol_simd(simd, self, skip_tab, &mut has_yaml_ws))
    }
    
    fn push_plain_run(&mut self, in_flow: bool, out: &mut Vec<u8>) -> usize {
        let mut n_chars = 0;
        while !is_blank_or_breakz(self.peekz(0)) && self.next_can_be_plain_scalar(in_flow) {
            n_chars += 1;
            out.push(self.peekz(0));

            self.skip(1);
            // Ensures that we are close to end we reset the head to start
            if self.pos == BUF_MAX_LEN - 1 {
                self.reset_head_to_start();
            } 
        }
        n_chars
    
    }
}

fn push_non_break_simd<T, S>(simd: S, src: &mut BufferedBytesSource<T>, vec: &mut Vec<u8>)
where
    T: Iterator<Item = u8>,
    S: Simd,
{
    simd.vectorize(
        // TODO RE-ENABLE #[inline(always)]
        || {
            
            while let Some(x) = src.get_chunk_buf() {
                let mut pos = 0;
                let mut chunks = x.chunks_exact(S::u8s::LEN);
                for chunk in &mut chunks {
                    let value = S::u8s::from_slice(simd, chunk);
                    let break_bitmask =
                        value.simd_eq(b'\r').to_bitmask() | value.simd_eq(b'\n').to_bitmask();
                    pos += break_bitmask.trailing_zeros() as usize;
                    if break_bitmask != 0 {
                        vec.extend_from_slice(&x[0..pos]);
                        src.pos = pos;
                        return;
                    }
                }

                vec.extend_from_slice(x);
                src.fill_buf_to_max();
            }
            let buf = src.get_buf();

            // we get the value from while loop above, or we search remaining buf
            let found_pos = buf.iter().position(|&c| is_break(c)).unwrap_or(buf.len());
            vec.extend_from_slice(&buf[..found_pos]);
            src.skip(found_pos);
        },
    );
}

fn skip_spaces_simd<T, S>(simd: S, src: &mut BufferedBytesSource<T>, max: usize) -> usize
where
    T: Iterator<Item = u8>,
    S: Simd,
    S: Simd,
{
    simd.vectorize(
        // TODO RE-ENABLE #[inline(always)]
        || {
            let mut n_chars = 0;

            while let Some(x) = src.get_chunk_buf() {
                let mut chunks = x.chunks_exact(S::u8s::LEN);
                for chunk in &mut chunks {
                    let value = S::u8s::from_slice(simd, chunk);
                    let ws_mask = value.simd_eq(b' ').to_bitmask();
                    let n = (ws_mask.trailing_ones() as usize).min(max);
                    n_chars += n;
                    if n < S::u8s::LEN || n_chars >= max {
                        src.pos += n;
                        return n_chars;
                    }
                }
                src.fill_buf_to_max();
            }

            let debug = src.get_buf();
            let slow_path = debug
                .iter()
                .position(|&c| c != b' ')
                .unwrap_or(debug.len())
                .min(max);

            src.skip(slow_path);
            n_chars += slow_path;
            n_chars
        },
    )
}

fn skip_ws_to_eol_simd<T, S>(
    simd: S,
    src: &mut BufferedBytesSource<T>,
    skip_tabs: bool,
    has_yaml_ws: &mut bool,
) -> (u32, Result<SkipTabs, &'static str>)
where
    T: Iterator<Item = u8>,
    S: Simd,
{
    simd.vectorize(
        // TODO RE-ENABLE #[inline(always)]
        || {
            let mut any_tabs = false;
            let mut bytes_consumed = 0u32;

            'break_finalize: while let Some(x) = src.get_chunk_buf() {
                let mut chunks = x.chunks_exact(S::u8s::LEN);
                for chunk in &mut chunks {
                    let value = S::u8s::from_slice(simd, chunk);

                    let spaces = value.simd_eq(b' ').to_bitmask();
                    let tabs = if skip_tabs {
                        value.simd_eq(b'\t').to_bitmask()
                    } else {
                        0
                    };

                    let n = (spaces | tabs).trailing_ones() as usize;
                    let prefix = if n == 64 { u64::MAX } else { (1 << n) - 1 };

                    *has_yaml_ws |= (spaces & prefix) != 0;
                    any_tabs |= (tabs & prefix) != 0;

                    // src.filled += n;
                    bytes_consumed += n as u32;
                    if n < S::u8s::LEN {
                        src.pos += n;
                        break 'break_finalize;
                    }
                }
                src.fill_buf_to_max();
            }

            shared_skip_ws_to_eol(src, skip_tabs, bytes_consumed, any_tabs, *has_yaml_ws)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::parsing::{Source};
    use crate::parsing::buffered_source::{BUF_MAX_LEN, BufferedBytesSource};
    use crate::parsing::scanner::SkipTabs;
    use alloc::vec::Vec;

    pub const BLOCK_MULTI_INPUT: &str = r"    word1
    # comment
    word2
";

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
    fn test_prop_push() {
        let src = "a:a:aa:::::a:::::a::aaaaa:a:a:a::";
        let mut source = BufferedBytesSource::new(src);
        let mut out = Vec::with_capacity(32);

        assert_eq!(source.peekz(0), b'a');
        source.push_plain_run(false, &mut out);
        assert_eq!(source.peekz(0), b':');
        assert_eq!(out, src.as_bytes()[0..src.len()-1])
    }

    #[test]
    fn test_skip_spaces() {
        let src = BLOCK_MULTI_INPUT;
        let mut source = BufferedBytesSource::new(src);

        assert_eq!(source.peekz(0), b' ');
        let skip = source.skip_spaces(usize::MAX);
        assert_eq!(skip, 4);
        assert_eq!(source.peekz(0), b'w');
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
        assert_eq!(source.filled, BUF_MAX_LEN);
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
