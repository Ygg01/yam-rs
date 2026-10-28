use proptest::prelude::*;
use yam_core::parsing::{BufferedBytesSource, Source, StrSource};

fn peekz(a: &str, n: usize, is_buf: bool) -> u8 {
    if is_buf {
        let src = BufferedBytesSource::new(a);
        src.peekz(n)
    } else {
        let src = StrSource::new(a);
        src.peekz(n)
    }
}

/// Runs `op` on the source and returns its result, followed by all bytes the source has left.
fn run_op<S: Source, R>(mut src: S, op: impl FnOnce(&mut S) -> R) -> (R, Vec<u8>) {
    let res = op(&mut src);
    let mut rest = Vec::new();
    while let Some(x) = src.peek_checked(0) {
        rest.push(x);
        src.skip(1);
    }
    (res, rest)
}

/// Source operations whose results must not depend on the source implementation.
#[derive(Clone, Copy, Debug)]
enum SourceOp {
    SkipSpaces(usize),
    PushPlainRun(bool),
    PushNonBreakz,
    SkipWsToEol { skip_tabs: bool, prev_ws: bool },
}

fn apply<S: Source>(src: S, op: SourceOp) -> ((usize, Vec<u8>), Vec<u8>) {
    run_op(src, |src| {
        let mut out = Vec::new();
        let n = match op {
            SourceOp::SkipSpaces(max) => src.skip_spaces(max),
            SourceOp::PushPlainRun(in_flow) => src.push_plain_run(in_flow, &mut out),
            SourceOp::PushNonBreakz => {
                src.push_non_breakz_chr(&mut out);
                out.len()
            }
            SourceOp::SkipWsToEol { skip_tabs, prev_ws } => {
                let (n, res) = src.skip_ws_to_eol(skip_tabs, prev_ws);
                out = format!("{res:?}").into_bytes();
                n as usize
            }
        };
        (n, out)
    })
}

fn source_op() -> impl Strategy<Value = SourceOp> {
    prop_oneof![
        prop_oneof![0..80usize, Just(usize::MAX)].prop_map(SourceOp::SkipSpaces),
        any::<bool>().prop_map(SourceOp::PushPlainRun),
        Just(SourceOp::PushNonBreakz),
        (any::<bool>(), any::<bool>())
            .prop_map(|(skip_tabs, prev_ws)| SourceOp::SkipWsToEol { skip_tabs, prev_ws }),
    ]
}

proptest! {
    #[test]
    fn test_add(a in r"[#\[\]\{\}\w]*", b in 0..32usize) {
        let buffered = peekz(&a, b, true);
        let stringed = peekz(&a, b, false);
        assert_eq!(buffered, stringed);
    }

    #[test]
    fn test_run_ops(a in r"[ \t\r\n\x00:#,\[\]\{\}ab]{0,100}", op in source_op()) {
        let buffered = apply(BufferedBytesSource::new(&a), op);
        let stringed = apply(StrSource::new(&a), op);
        prop_assert_eq!(buffered, stringed);
    }

    #[test]
    fn test_run_ops_long_runs(
        a in r"( {0,70}|[a-z:]{0,70}|[a-z ]{0,70})[\r\n\x00 :,\]]?[a-z ]{0,10}",
        op in source_op(),
    ) {
        let buffered = apply(BufferedBytesSource::new(&a), op);
        let stringed = apply(StrSource::new(&a), op);
        prop_assert_eq!(buffered, stringed);
    }
}
