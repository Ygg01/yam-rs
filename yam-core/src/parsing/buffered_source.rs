use crate::parsing::Source;
use crate::parsing::char_utils::is_break;
use crate::parsing::scanner::SkipTabs;
use crate::parsing::source::shared_skip_ws_to_eol;
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
    #[allow(dead_code)]
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
                    // SAFETY: This will always be in buffer len
                    unsafe {
                        self.buf.get_unchecked_mut(x).write(byt);
                    }
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

#[allow(dead_code)]
impl<'a> BufferedBytesSource<Copied<Iter<'a, u8>>> {
    pub fn from_bytes(input: &'a [u8]) -> Self {
        let mut x = Self {
            input: input.iter().copied(),
            buf: [MaybeUninit::uninit(); MAX_LEN],
            len: 0,
        };
        x.fill_buf_to_max();
        x
    }

    pub fn from_str(input: &'a str) -> Self {
        Self::from_bytes(input.as_bytes())
    }
}
unsafe impl<T: Iterator<Item = u8>> Source for BufferedBytesSource<T> {
    unsafe fn peek_unsafe(&self, n: usize) -> u8 {
        unsafe { self.buf[n].assume_init() }
    }

    fn peek_checked(&self, n: usize) -> Option<u8> {
        debug_assert!(n < self.buf_max_len());
        if n >= self.len {
            return None;
        }
        unsafe { Some(self.peek_unsafe(n)) }
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

    fn buf_max_len(&self) -> usize {
        MAX_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.len == 0
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
        let found_pos = pos.unwrap_or(buf.iter().position(|&c| is_break(c)).unwrap_or(0));
        vec.extend_from_slice(&buf[..found_pos]);
        self.skip(found_pos);
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

    #[allow(clippy::cast_possible_truncation)]
    fn skip_ws_to_eol(
        &mut self,
        skip_tab: bool,
        mut has_yaml_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>) {
        let any_tabs = false;
        let mut consumed_bytes = 0u32;
        let mut consume = 0u32;
        let mut skip_tabs_res = SkipTabs::No;

        let low_nib_mask = U8X16::splat(0xF);
        let high_nib_mask = U8X16::splat(0x7F);
        let ws_flag = 0x04 + u8::from(skip_tab);

        while let Some(x) = self.get_max_buf() {
            consume = 0;
            let (v0, v1) = U8X32::from_array(x).split();

            let v_v0 = LOW_NIBBLE_WS.swizzle(v0 & low_nib_mask)
                & HIGH_NIBBLE_WS.swizzle((v0 >> 4) & high_nib_mask);
            let v_v1 = LOW_NIBBLE_WS.swizzle(v1 & low_nib_mask)
                & HIGH_NIBBLE_WS.swizzle((v1 >> 4) & high_nib_mask);

            let sp = !U8X32::merge(v_v0 & ws_flag, v_v1 & ws_flag)
                .comp(0)
                .to_bitmask();
            let nl = !U8X32::merge(v_v0 & 0x02, v_v1 & 0x02).comp(0).to_bitmask();
            let hash = !U8X32::merge(v_v0 & 0x08, v_v1 & 0x08).comp(0).to_bitmask();

            let end_of_line = (hash & !(sp << 1)) | nl;

            has_yaml_ws |= sp != 0;

            if end_of_line != 0 {
                consume = end_of_line.trailing_zeros();
                consumed_bytes += consume;
                skip_tabs_res = SkipTabs::Result {
                    any_tabs,
                    has_yaml_ws,
                };
                break;
            }

            self.skip(self.buf_max_len());
            consumed_bytes += self.buf_max_len() as u32;
        }
        self.skip(consume as usize);

        if matches!(skip_tabs_res, SkipTabs::Result { .. } | SkipTabs::Yes) {
            return (consumed_bytes, Ok(skip_tabs_res));
        }

        shared_skip_ws_to_eol(self, skip_tab, consumed_bytes, any_tabs, has_yaml_ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::Source;
    use crate::parsing::buffered_source::{BufferedBytesSource, MAX_LEN};
    use alloc::vec::Vec;

    #[test]
    fn test_create() {
        let source = BufferedBytesSource::from_bytes(b"Hello, world!");
        assert_eq!(source.len, 13);
        assert_eq!(source.peekz(0), b'H');
        assert_eq!(source.peekz(1), b'e');
    }

    #[test]
    fn test_skip() {
        let mut source = BufferedBytesSource::from_bytes(b"Hello, world!");
        assert_eq!(source.len, 13);
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
        let mut source = BufferedBytesSource::from_bytes(
            br#"Lorem ipsum dolor sit amet, 
            consectetur adipiscing elit. Sed dui nulla, consectetur in pretium sit amet, 
            ornare vitae erat. Aenean bibendum arcu et risus auctor, 
            nec finibus arcu finibus. Integer ut congue metus, non hendrerit nunc. 
            Ut ornare efficitur nisl, sed ullamcorper risus feugiat at. 
            Aenean ut mi a nulla pellentesque aliquet quis vitae lorem. Vestibulum semper elit"#,
        );
        assert_eq!(source.len, MAX_LEN);
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
        let mut source = BufferedBytesSource::from_bytes(b"");
        assert_eq!(source.len, 0);
        assert_eq!(source.peekz(0), b'\0');
        assert_eq!(source.peekz(1), b'\0');

        source.skip(130);
        assert_eq!(source.peekz(0), b'\0');
        assert_eq!(source.peekz(1), b'\0');
    }

    #[test]
    fn test_push_breakz() {
        let mut source = BufferedBytesSource::from_bytes(b"Lorem ipsum dolor sit amet,
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
        let mut source = BufferedBytesSource::from_str(YAML_WS_TABS);
        assert_eq!(source.peekz(0), b' ');
        let res1 = source.skip_ws_to_eol(true, false);
        assert!(res1.1.is_ok());
        assert_eq!(res1.0, 83);

        let skip_tabs = res1.1.unwrap();
        assert!(skip_tabs.found_tabs());
        assert!(skip_tabs.has_valid_yaml_ws());

        let mut source = BufferedBytesSource::from_str(YAML_WS_TABS);
        assert_eq!(source.peekz(0), b' ');
        let res2 = source.skip_ws_to_eol(false, false);
        assert!(res2.1.is_ok());
        assert_eq!(res2.0, 72);
    }
}
