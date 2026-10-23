use std::fmt::Write;
use yam_core::parsing::{Event, Parser, Source};

pub const X1_R52L_INPUT: &str = r#"
{ 
    # comments
    top1: [item, item] 
} "#;

pub const X1_R52L_EXPECTED: &str = r#"
+STR Span { start: Marker { pos: 0, col: 1, line: 1 }, end: Marker { pos: 0, col: 1, line: 1 } }
+DOC Span { start: Marker { pos: 1, col: 1, line: 2 }, end: Marker { pos: 3, col: 3, line: 2 } }
+MAP Span { start: Marker { pos: 1, col: 1, line: 2 }, end: Marker { pos: 3, col: 3, line: 2 } }
:SCA Span { start: Marker { pos: 23, col: 5, line: 4 }, end: Marker { pos: 27, col: 9, line: 4 } }
+SEQ Span { start: Marker { pos: 29, col: 11, line: 4 }, end: Marker { pos: 30, col: 12, line: 4 } }
:SCA Span { start: Marker { pos: 30, col: 12, line: 4 }, end: Marker { pos: 34, col: 16, line: 4 } }
:SCA Span { start: Marker { pos: 36, col: 18, line: 4 }, end: Marker { pos: 40, col: 22, line: 4 } }
-SEQ Span { start: Marker { pos: 40, col: 22, line: 4 }, end: Marker { pos: 42, col: 24, line: 4 } }
-MAP Span { start: Marker { pos: 43, col: 1, line: 5 }, end: Marker { pos: 45, col: 3, line: 5 } }
-DOC Span { start: Marker { pos: 45, col: 1, line: 6 }, end: Marker { pos: 45, col: 1, line: 6 } }
-STR Span { start: Marker { pos: 45, col: 1, line: 6 }, end: Marker { pos: 45, col: 1, line: 6 } }"#;

#[test]
fn run_docs() {
    assert_eq_spanned_events(X1_R52L_INPUT, X1_R52L_EXPECTED);
}

#[doc(hidden)]
pub fn assert_eq_spanned_events(input: &str, events: &str) {
    let mut line = String::with_capacity(events.len() + 1);
    line.push('\n');
    let mut parser = Parser::new_from_str(input);
    write_str_from_event(&mut line, &mut parser);
    assert_eq!(line.trim_end(), events);
}

pub fn write_str_from_event<T: Source>(line: &mut String, parser: &mut Parser<T>) {
    while let Some(Ok((ev, span))) = parser.next() {
        match ev {
            Event::Nothing => {}
            Event::StreamStart => writeln!(line, "+STR {:?}", span).unwrap(),
            Event::StreamEnd => writeln!(line, "-STR {:?}", span).unwrap(),
            Event::DocumentStart(_) => writeln!(line, "+DOC {:?}", span).unwrap(),
            Event::DocumentEnd => writeln!(line, "-DOC {:?}", span).unwrap(),
            Event::Alias(_) => writeln!(line, ":ALI {:?}", span).unwrap(),
            Event::Comment(_) => writeln!(line, ":COM {:?}", span).unwrap(),
            Event::Scalar(_) => writeln!(line, ":SCA {:?}", span).unwrap(),
            Event::SequenceStart(_, _) => writeln!(line, "+SEQ {:?}", span).unwrap(),
            Event::SequenceEnd => writeln!(line, "-SEQ {:?}", span).unwrap(),
            Event::MappingStart(_, _) => writeln!(line, "+MAP {:?}", span).unwrap(),
            Event::MappingEnd => writeln!(line, "-MAP {:?}", span).unwrap(),
        }
    }
}
