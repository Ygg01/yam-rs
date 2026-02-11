use crate::Source;
use crate::saphyr_tokenizer::char_utils::is_break;
use crate::saphyr_tokenizer::scanner::SkipTabs;
use crate::saphyr_tokenizer::source::shared_skip_ws_to_eol;
use crate::util::{BitOps, HIGH_NIBBLE_WS, LOW_NIBBLE_WS, U8X16, U8X32};
use alloc::vec::Vec;
use core::iter::Copied;
use core::mem::{MaybeUninit, transmute};
use core::slice;
use core::slice::Iter;

const MAX_LEN: usize = 32;

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

    pub fn get_max_buf(&self) -> Option<[u8; MAX_LEN]> {
        if self.len < self.buf_max_len() {
            return None;
        }
        let buf = unsafe { transmute::<[MaybeUninit<u8>; MAX_LEN], [u8; MAX_LEN]>(self.buf) };
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

    pub fn from_str(input: &'a str) -> Self {
        Self::from_bstr(input.as_bytes())
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
        if n == 0 {
            return;
        }

        let consume = n.min(self.len);
        let skip = n.saturating_sub(self.buf_max_len() + 1);

        if skip > 0 {
            self.input.nth(skip);
        }

        self.buf.copy_within(consume..self.len, 0);

        let new_len = self.len.saturating_sub(consume);
        self.len = new_len;
        self.fill_buf_to_max();
    }

    fn buf_max_len(&self) -> usize {
        MAX_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.len == 0
    }

    //noinspection ALL
    fn skip_ws_to_eol(&mut self, skip_tab: bool) -> (u32, Result<SkipTabs, &'static str>) {
        let mut has_yaml_ws = false;
        let mut any_tabs = false;
        let mut consumed_bytes = 0u32;
        let mut skip_tabs_res = SkipTabs::No;

        let low_nib_mask = U8X16::splat(0xF);
        let high_nib_mask = U8X16::splat(0x7F);
        let ws_flag = 0x04 + (skip_tab as u8);

        while let Some(x) = self.get_max_buf() {
            let (v0, v1) = U8X32::from_array(x).split();

            let v_v0 = LOW_NIBBLE_WS.swizzle(v0 & low_nib_mask)
                & HIGH_NIBBLE_WS.swizzle((v0 >> 4) & high_nib_mask);
            let v_v1 = LOW_NIBBLE_WS.swizzle(v1 & low_nib_mask)
                & HIGH_NIBBLE_WS.swizzle((v1 >> 4) & high_nib_mask);

            let v0_flag = (v_v0 & ws_flag).comp(0);
            let sp = U8X32::merge(v_v0 & ws_flag, v_v1 & ws_flag)
                .comp(0)
                .to_bitmask();
            let nl = U8X32::merge(v_v0 & 0x02, v_v1 & 0x02).comp(0).to_bitmask();
            let hash = U8X32::merge(v_v0 & 0x08, v_v1 & 0x08).comp(0).to_bitmask();

            let invalid_comment = hash & !(sp << 1);
            if invalid_comment != 0 {
                let consume = (invalid_comment | nl).trailing_zeros();
                consumed_bytes += consume;
                self.skip(consume as usize);
                skip_tabs_res = SkipTabs::Result {
                    any_tabs,
                    has_yaml_ws,
                };
                break;
            }

            has_yaml_ws |= sp != 0;

            if sp != 0 {
                let consume = nl.trailing_zeros().saturating_sub(sp.trailing_zeros());
                consumed_bytes += consume;
                skip_tabs_res = SkipTabs::Result {
                    any_tabs,
                    has_yaml_ws,
                };
                break;
            }

            self.skip(self.buf_max_len())
        }

        if matches!(skip_tabs_res, SkipTabs::Result { .. } | SkipTabs::Yes) {
            return (consumed_bytes, Ok(skip_tabs_res));
        }

        shared_skip_ws_to_eol(self, skip_tab, any_tabs, has_yaml_ws)
    }

    fn next_is_z(&self) -> bool {
        self.buf_is_empty()
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
            self.skip(MAX_LEN)
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
    use crate::saphyr_tokenizer::buffered_source::{BufferedBytesSource, MAX_LEN};
    use alloc::vec::Vec;

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
        assert_eq!(source.len, MAX_LEN);
        assert_eq!(source.peek(), b'L');
        assert_eq!(source.peek_n1(), b'o');

        // Skip characters to ornare vitae
        source.skip(131);
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

    #[test]
    fn test_push_breakz() {
        let mut source = BufferedBytesSource::from_bstr(b"Lorem ipsum dolor sit amet,
                                                        consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet,
                                                        ornare vitae erat. Aenean bibendum arcu et risus auctor,");
        let mut dest = Vec::new();
        source.push_non_breakz_chr(&mut dest);
        assert_eq!(source.peek(), b'\n');
        assert_eq!(
            str::from_utf8(&dest).unwrap(),
            "Lorem ipsum dolor sit amet,"
        );

        // Skip newline
        source.skip(1);

        dest.clear();
        source.push_non_breakz_chr(&mut dest);
        assert_eq!(source.peek(), b'\n');
        assert_eq!(
            str::from_utf8(&dest).unwrap(),
            "                                                        consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet,"
        );
    }
}
