use proptest::prelude::*;
use yam_core::parsing::{BufferedBytesSource, Source, StrSource};

proptest! {
    #[test]
    fn test_prop_peekz(
        a in r"[#\[\]\{\}\w]*",
        b in 0..32usize,
    ) {
        let buffered = BufferedBytesSource::new(&a);
        let stringed = StrSource::new(&a);
        let peekz_buff = buffered.peekz(b);
        let peekz_strn = stringed.peekz(b);
        prop_assert_eq!(peekz_buff, peekz_strn);
    }


    #[test]
    fn test_prop_skip_space(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        b in 0..32usize,
    ) {
        let mut buffered = BufferedBytesSource::new(&a);
        let mut stringed = StrSource::new(&a);
        let skip_buffered = buffered.skip_spaces(b);
        let skip_stringed = stringed.skip_spaces(b);
        prop_assert_eq!(skip_buffered, skip_stringed);
    }


    #[test]
    fn test_prop_push_plain_run(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        in_flow in any::<bool>(),
    ) {
        let mut stringed = StrSource::new(&a);
        let mut stringed_vec = Vec::new();
        let skip_stringed = stringed.push_plain_run(in_flow, &mut stringed_vec);

        let mut buffered = BufferedBytesSource::new(&a);
        let mut buffered_vec = Vec::new();
        let skip_buffered = buffered.push_plain_run(in_flow, &mut buffered_vec);

        prop_assert_eq!(skip_buffered, skip_stringed);
        prop_assert_eq!(stringed_vec, buffered_vec);
    }

    #[test]
    fn test_prop_push_non_breakz_chr(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
    ) {
        let mut stringed = StrSource::new(&a);
        let mut stringed_vec = Vec::new();
        stringed.push_non_breakz_chr(&mut stringed_vec);

        let mut buffered = BufferedBytesSource::new(&a);
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
        let mut stringed = StrSource::new(&a);
        let str_res = stringed.skip_ws_to_eol(skip_tabs, prev_ws);

        let mut buffered = BufferedBytesSource::new(&a);
        let buf_res = buffered.skip_ws_to_eol(skip_tabs, prev_ws);

        prop_assert_eq!(str_res, buf_res);
    }

}
