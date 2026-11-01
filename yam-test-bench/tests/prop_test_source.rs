use proptest::prelude::*;
use yam_core::parsing::{BufBytesSource, InMemSource, Source};

/// Inputs long enough to cross the internal buffer of `BufferedBytesSource`.
const LONG_INPUT: &str =
    r"[a-z ]{100,200}( {0,140}|[a-z:]{0,140}|[a-z ]{0,140})[\r\n\x00 :,\]]?[a-z ]{0,10}";

fn assert_same_head(buffered: &impl Source, stringed: &impl Source) -> Result<(), TestCaseError> {
    for n in 0..4 {
        prop_assert_eq!(buffered.peekz(n), stringed.peekz(n), "peekz({}) differs", n);
    }
    Ok(())
}

proptest! {
    #[test]
    fn test_prop_long_skip_spaces(
        a in LONG_INPUT,
        skip in 0..200usize,
        max in 0..200usize,
    ) {
        let mut buffered = BufBytesSource::new(&a);
        let mut stringed = InMemSource::new(&a);
        let skip = skip.min(a.len());
        buffered.skip(skip);
        stringed.skip(skip);
        assert_same_head(&buffered, &stringed)?;
        prop_assert_eq!(buffered.skip_spaces(max), stringed.skip_spaces(max));
        assert_same_head(&buffered, &stringed)?;
    }

    #[test]
    fn test_prop_long_push_plain_run(
        a in LONG_INPUT,
        skip in 0..200usize,
        in_flow in any::<bool>(),
    ) {
        let mut buffered = BufBytesSource::new(&a);
        let mut stringed = InMemSource::new(&a);
        let skip = skip.min(a.len());
        buffered.skip(skip);
        stringed.skip(skip);
        let mut buffered_vec = Vec::new();
        let mut stringed_vec = Vec::new();
        prop_assert_eq!(
            buffered.push_plain_run(in_flow, &mut buffered_vec),
            stringed.push_plain_run(in_flow, &mut stringed_vec)
        );
        prop_assert_eq!(buffered_vec, stringed_vec);
        assert_same_head(&buffered, &stringed)?;
    }

    #[test]
    fn test_prop_long_push_non_breakz_chr(
        a in LONG_INPUT,
        skip in 0..200usize,
    ) {
        let mut buffered = BufBytesSource::new(&a);
        let mut stringed = InMemSource::new(&a);
        let skip = skip.min(a.len());
        buffered.skip(skip);
        stringed.skip(skip);
        let mut buffered_vec = Vec::new();
        let mut stringed_vec = Vec::new();
        buffered.push_non_breakz_chr(&mut buffered_vec);
        stringed.push_non_breakz_chr(&mut stringed_vec);
        prop_assert_eq!(buffered_vec, stringed_vec);
        assert_same_head(&buffered, &stringed)?;
    }

    #[test]
    fn test_prop_long_skip_ws_to_eol(
        a in LONG_INPUT,
        skip in 0..200usize,
        skip_tabs in any::<bool>(),
        prev_ws in any::<bool>(),
    ) {
        let mut buffered = BufBytesSource::new(&a);
        let mut stringed = InMemSource::new(&a);
        let skip = skip.min(a.len());
        buffered.skip(skip);
        stringed.skip(skip);
        prop_assert_eq!(
            buffered.skip_ws_to_eol(skip_tabs, prev_ws),
            stringed.skip_ws_to_eol(skip_tabs, prev_ws)
        );
        assert_same_head(&buffered, &stringed)?;
    }


    #[test]
    fn test_prop_peekz(
        a in r"[#\[\]\{\}\w]*",
        b in 0..32usize,
    ) {
        let buffered = BufBytesSource::new(&a);
        let stringed = InMemSource::new(&a);
        let peekz_buff = buffered.peekz(b);
        let peekz_strn = stringed.peekz(b);
        prop_assert_eq!(peekz_buff, peekz_strn);
    }


    #[test]
    fn test_prop_skip_space(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        b in 0..32usize,
    ) {
        let mut buffered = BufBytesSource::new(&a);
        let mut stringed = InMemSource::new(&a);
        let skip_buffered = buffered.skip_spaces(b);
        let skip_stringed = stringed.skip_spaces(b);
        prop_assert_eq!(skip_buffered, skip_stringed);
    }


    #[test]
    fn test_prop_push_plain_run(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        in_flow in any::<bool>(),
    ) {
        let mut stringed = InMemSource::new(&a);
        let mut stringed_vec = Vec::new();
        let skip_stringed = stringed.push_plain_run(in_flow, &mut stringed_vec);

        let mut buffered = BufBytesSource::new(&a);
        let mut buffered_vec = Vec::new();
        let skip_buffered = buffered.push_plain_run(in_flow, &mut buffered_vec);

        prop_assert_eq!(skip_buffered, skip_stringed);
        prop_assert_eq!(stringed_vec, buffered_vec);
    }

    #[test]
    fn test_prop_push_non_breakz_chr(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
    ) {
        let mut stringed = InMemSource::new(&a);
        let mut stringed_vec = Vec::new();
        stringed.push_non_breakz_chr(&mut stringed_vec);

        let mut buffered = BufBytesSource::new(&a);
        let mut buffered_vec = Vec::new();
        buffered.push_non_breakz_chr(&mut buffered_vec);

        prop_assert_eq!(stringed_vec, buffered_vec);
    }

    #[test]
    fn test_prop_skip_ws_to_eol(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        skip_tabs in any::<bool>(),
        prev_ws in any::<bool>(),
    ) {
        let mut stringed = InMemSource::new(&a);
        let str_res = stringed.skip_ws_to_eol(skip_tabs, prev_ws);

        let mut buffered = BufBytesSource::new(&a);
        let buf_res = buffered.skip_ws_to_eol(skip_tabs, prev_ws);

        prop_assert_eq!(str_res, buf_res);
    }

}
