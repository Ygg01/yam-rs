use proptest::prelude::*;
use yam_core::parsing::{BufferedBytesSource, Source, StrSource};

fn peekz(a: &str, n: usize, is_buf: bool) -> u8 {
    if is_buf {
        let src = BufferedBytesSource::new_from_str(a);
        src.peekz(n)
    } else {
        let src = StrSource::new(a);
        src.peekz(n)
    }
}

proptest! {
    #[test]
    fn test_add(a in r"[#\[\]\{\}\w]*", b in 0..32usize) {
        let buffered = peekz(&a, b, true);
        let stringed = peekz(&a, b, false);
        assert_eq!(buffered, stringed);
    }
}
