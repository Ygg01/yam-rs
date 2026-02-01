pub mod consts;

use std::borrow::Cow;
use std::fmt::Write;
use yam_core::saphyr_tokenizer::{ScalarValue, Source, Tag};
use yam_core::{Parser, SaphyrEvent};

///
/// Assert that in for given input, the parser generates expected set of events
///
/// # Panics
///
///    Function panics if there is a difference between expected events string and one generated
///    from the input.
pub fn assert_eq_event_case_saph(input: &str, events: &str) {
    let mut line = String::new();
    let mut parser = Parser::new_from_str(input);

    write_str_from_event(&mut line, &mut parser, false);
    let expected_err = events.ends_with("ERR");
    let actual_err = events.ends_with("ERR");
    assert_eq!(actual_err, expected_err);
    if !expected_err {
        assert_eq!(line, unescape_text(events), "Error in case: {input}");
    }
}

pub fn write_str_from_event<T: Source>(
    line: &mut String,
    parser: &mut Parser<T>,
    emit_stream_token: bool,
) {
    while let Some(Ok((ev, _))) = parser.next() {
        let _ = match ev {
            SaphyrEvent::StreamStart if emit_stream_token => write!(line, "+STR"),
            SaphyrEvent::StreamEnd if emit_stream_token => write!(line, "\n-STR"),
            SaphyrEvent::DocumentStart(_) => write!(line, "\n+DOC"),
            SaphyrEvent::DocumentEnd => write!(line, "\n-DOC"),
            SaphyrEvent::Alias(anchor_id) => {
                let anchor = parser
                    .get_anchor(anchor_id)
                    .map_or(String::default(), |x| x.to_string());
                write!(line, "\n=ALI *{}", anchor)
            }
            SaphyrEvent::Scalar(ScalarValue {
                value,
                scalar_type,
                anchor_id,
                tag,
            }) => {
                let anchor = extract_anchor(parser, anchor_id);
                let tag_info = extract_tag_full(tag);
                write!(line, "\n=VAL{anchor}{tag_info} {scalar_type}{value}")
            }
            SaphyrEvent::SequenceStart(anchor_id, tag) => {
                let tag_info = extract_tag_full(tag);
                let anchor = extract_anchor(parser, anchor_id);
                write!(line, "\n+SEQ{anchor}{tag_info}")
            }
            SaphyrEvent::SequenceEnd => write!(line, "\n-SEQ"),
            SaphyrEvent::MappingStart(anchor_id, tag) => {
                let tag_info = extract_tag_full(tag);
                let anchor = extract_anchor(parser, anchor_id);
                write!(line, "\n+MAP{anchor}{tag_info}")
            }
            SaphyrEvent::MappingEnd => write!(line, "\n-MAP"),
            _ => write!(line, ""),
        };
    }
    if let Some(Err(err)) = parser.next() {
        line.push_str("\nERR");
        if emit_stream_token {
            write!(line, "{:?}", err).unwrap()
        }
    }
    if emit_stream_token {
        line.push('\n');
    }
}

fn extract_tag_full(tag: Option<Cow<Tag>>) -> String {
    if let Some(x) = tag {
        let cap = x.suffix.len() + x.handle.len() + 5;
        let mut sb = String::with_capacity(cap);
        sb.push_str(" <");
        sb.push_str(&x.handle);
        sb.push_str(&x.suffix);
        sb.push('>');
        sb
    } else {
        String::default()
    }
}

fn extract_anchor<T: Source>(parser: &Parser<T>, anchor_id: usize) -> String {
    let anchor = if let Some(cow) = parser.get_anchor(anchor_id) {
        let mut out = String::with_capacity(cow.len() + 2);
        write!(out, " &{}", cow).unwrap();
        out
    } else {
        String::default()
    };
    anchor
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EscapeState {
    Default,
    Slash,
}

pub fn unescape_text(text: &str) -> String {
    let mut output = Vec::with_capacity(text.len());
    let chars = text.as_bytes();
    let mut state = EscapeState::Default;
    for char in chars {
        match (char, state) {
            (b'\\', EscapeState::Default) => {
                state = EscapeState::Slash;
            }
            (b'n', EscapeState::Slash) => {
                state = EscapeState::Default;
                output.push(b'\n');
            }
            (b'r', EscapeState::Slash) => {
                state = EscapeState::Default;
                output.push(b'\r');
            }
            (b't', EscapeState::Slash) => {
                state = EscapeState::Default;
                output.push(b'\t');
            }
            (b'b', EscapeState::Slash) => {
                state = EscapeState::Default;
                output.push(b'\x08');
            }
            (b'\\', EscapeState::Slash) => {
                state = EscapeState::Default;
                output.push(b'\\');
            }

            (_, EscapeState::Default | EscapeState::Slash) => output.push(*char),
        }
    }

    String::from_utf8(output).unwrap()
}
