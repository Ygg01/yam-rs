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

pub const EVENT_BUF: &str = r#"
+DOC Span [0..0] From (row: 1, col 1) To (row: 1, col 1)
+MAP Span [0..1] From (row: 1, col 1) To (row: 1, col 2)
VAL: Span [1..2] From (row: 1, col 2) To (row: 1, col 3)
VAL: Span [4..5] From (row: 1, col 5) To (row: 1, col 6)
VAL: Span [7..8] From (row: 1, col 8) To (row: 1, col 9)
VAL: Span [10..13] From (row: 1, col 11) To (row: 1, col 14)
-MAP Span [13..14] From (row: 1, col 14) To (row: 1, col 15)
-DOC Span [14..14] From (row: 2, col 1) To (row: 2, col 1)"#;

#[test]
fn test_span() {
    let input = "{a: b, c: aaa}";
    let mut spanned_buf = SpannedEventStr {
        buf: String::with_capacity(200),
    };
    let mut x = Parser::new_from_str(input);
    let _ = x.load(&mut spanned_buf, true);

    assert_eq!(spanned_buf.buf.trim(), EVENT_BUF.trim())
}
