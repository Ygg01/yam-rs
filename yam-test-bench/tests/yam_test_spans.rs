use rstest::rstest;
use std::fmt::Write;
use yam_core::parsing::{Event, Parser, SpannedEventReceiver};
use yam_core::prelude::Span;

struct SpannedEventStr {
    buf: String,
}

impl<'input> SpannedEventReceiver<'input> for SpannedEventStr {
    fn on_event(&mut self, ev: Event<'input>, span: Span) {
        let _ = match ev {
            Event::DocumentStart(_) => {
                writeln!(self.buf, "+DOC {}", span)
            }
            Event::DocumentEnd => {
                writeln!(self.buf, "-DOC {}", span)
            }
            Event::Alias(_) => {
                writeln!(self.buf, "ALI: {}", span)
            }
            Event::Comment(_) => {
                write!(self.buf, "COM: {}", span)
            }
            Event::Scalar(_) => {
                writeln!(self.buf, "VAL: {}", span)
            }
            Event::SequenceStart(_, _) => {
                writeln!(self.buf, "+SEQ {}", span)
            }
            Event::SequenceEnd => {
                writeln!(self.buf, "-SEQ {}", span)
            }
            Event::MappingStart(_, _) => {
                writeln!(self.buf, "+MAP {}", span)
            }
            Event::MappingEnd => {
                writeln!(self.buf, "-MAP {}", span)
            }
            // Ignored
            Event::Nothing | Event::StreamStart | Event::StreamEnd => Ok(()),
        };
    }
}

pub const FLOW_MAP_INPUT: &str = "{a: b, c: aaa}";

pub const FLOW_MAP_EVENTS: &str = r#"
+DOC Span [0..0] From (row: 1, col 1) To (row: 1, col 1)
+MAP Span [0..1] From (row: 1, col 1) To (row: 1, col 2)
VAL: Span [1..2] From (row: 1, col 2) To (row: 1, col 3)
VAL: Span [4..5] From (row: 1, col 5) To (row: 1, col 6)
VAL: Span [7..8] From (row: 1, col 8) To (row: 1, col 9)
VAL: Span [10..13] From (row: 1, col 11) To (row: 1, col 14)
-MAP Span [13..14] From (row: 1, col 14) To (row: 1, col 15)
-DOC Span [14..14] From (row: 1, col 15) To (row: 1, col 15)"#;

pub const BLOCK_MAP_INPUT: &str = "---
a: x
c: aaa
...";

pub const BLOCK_MAP_EVENTS: &str = r#"
+DOC Span [0..3] From (row: 1, col 1) To (row: 1, col 4)
+MAP Span [4..4] From (row: 2, col 1) To (row: 2, col 1)
VAL: Span [4..5] From (row: 2, col 1) To (row: 2, col 2)
VAL: Span [7..8] From (row: 2, col 4) To (row: 2, col 5)
VAL: Span [9..10] From (row: 3, col 1) To (row: 3, col 2)
VAL: Span [12..15] From (row: 3, col 4) To (row: 3, col 7)
-MAP Span [16..16] From (row: 4, col 1) To (row: 4, col 1)
-DOC Span [16..19] From (row: 4, col 1) To (row: 4, col 4)"#;

pub const X1_NULL_MAP_INPUT: &str = r#"
a: 'aaa'
b: !!null
"#;

pub const X1_NULL_MAP_EVENTS: &str = r#"
+DOC Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
+MAP Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
VAL: Span [1..2] From (row: 2, col 1) To (row: 2, col 2)
VAL: Span [4..9] From (row: 2, col 4) To (row: 2, col 9)
VAL: Span [10..11] From (row: 3, col 1) To (row: 3, col 2)
VAL: Span [20..20] From (row: 4, col 1) To (row: 4, col 1)
-MAP Span [20..20] From (row: 4, col 1) To (row: 4, col 1)
-DOC Span [20..20] From (row: 4, col 1) To (row: 4, col 1)"#;

pub const X2_NULL_MAP_INPUT: &str = r#"
a: 'aaa'
b: !!null null
"#;

pub const X2_NULL_MAP_EVENTS: &str = r#"
+DOC Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
+MAP Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
VAL: Span [1..2] From (row: 2, col 1) To (row: 2, col 2)
VAL: Span [4..9] From (row: 2, col 4) To (row: 2, col 9)
VAL: Span [10..11] From (row: 3, col 1) To (row: 3, col 2)
VAL: Span [20..24] From (row: 3, col 11) To (row: 3, col 15)
-MAP Span [25..25] From (row: 4, col 1) To (row: 4, col 1)
-DOC Span [25..25] From (row: 4, col 1) To (row: 4, col 1)"#;

pub const NESTED_INPUT: &str = r#"
a: !!vec [2, 3]"#;

pub const NESTED_EVENTS: &str = r#"
+DOC Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
+MAP Span [1..1] From (row: 2, col 1) To (row: 2, col 1)
VAL: Span [1..2] From (row: 2, col 1) To (row: 2, col 2)
+SEQ Span [10..11] From (row: 2, col 10) To (row: 2, col 11)
VAL: Span [11..12] From (row: 2, col 11) To (row: 2, col 12)
VAL: Span [14..15] From (row: 2, col 14) To (row: 2, col 15)
-SEQ Span [15..16] From (row: 2, col 15) To (row: 2, col 16)
-MAP Span [16..16] From (row: 2, col 16) To (row: 2, col 16)
-DOC Span [16..16] From (row: 2, col 16) To (row: 2, col 16)"#;

#[rstest]
#[case::span_flow_map(FLOW_MAP_INPUT, FLOW_MAP_EVENTS)]
#[case::span_block_map(BLOCK_MAP_INPUT, BLOCK_MAP_EVENTS)]
#[case::span_null_map_1(X1_NULL_MAP_INPUT, X1_NULL_MAP_EVENTS)]
#[case::span_null_map_2(X2_NULL_MAP_INPUT, X2_NULL_MAP_EVENTS)]
#[case::span_null_map_2(NESTED_INPUT, NESTED_EVENTS)]
fn test_span(#[case] input: &str, #[case] expected_event: &str) {
    let mut spanned_buf = SpannedEventStr {
        buf: String::with_capacity(200),
    };
    let mut parser = Parser::new_from_str(input);
    let _ = parser.load(&mut spanned_buf, true);

    assert_eq!(spanned_buf.buf.trim(), expected_event.trim())
}
