pub mod consts;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Write;
use yam_core::parsing::BufBytesSource;
use yam_core::parsing::Event;
use yam_core::parsing::{ScalarValue, Source, YamParser};
use yam_core::prelude::Tag;

#[derive(Copy, Clone)]
pub enum ParsingOptions {
    InMem,
    Buff,
}

pub const PAD: &str = "                                               ";

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub uid: u64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum OneEnumToHoldThemAll {
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Char(char),
    Option(Option<bool>),
    UnitVariant,
    TupleVariant(i32, bool),
    StructVariant { uid: u64 },
    NewTypeVariant(Person),
    VariantTuple((i8, i8)),
    VariantSeq(Vec<String>),
}

#[doc(hidden)]
pub fn assert_eq_event_case_with_opts(
    input: &str,
    events: &str,
    parsing_options: ParsingOptions,
    emit_opt: &FormatOpts,
) {
    let mut line = String::with_capacity(events.len());
    match parsing_options {
        ParsingOptions::InMem => {
            let mut parser = YamParser::new_from_str(input);
            write_str_from_event(&mut line, &mut parser, emit_opt);
        }
        ParsingOptions::Buff => {
            let mut parser = YamParser::new(BufBytesSource::new_from_bytes(input.as_bytes()));
            write_str_from_event(&mut line, &mut parser, emit_opt);
        }
    }

    let expected_err = events.ends_with("ERR");
    let actual_err = events.ends_with("ERR");
    assert_eq!(actual_err, expected_err);
    if !expected_err {
        assert_eq!(line, unescape_text(events), "Error in case: {input}");
    }
}

#[derive(Default)]
pub enum FormatOpts {
    #[default]
    YamlFormat,
    InlineTest,
    InlineTestDetailed,
}

impl FormatOpts {
    #[must_use]
    pub fn emit_err(&self) -> bool {
        match self {
            FormatOpts::YamlFormat | FormatOpts::InlineTest => false,
            FormatOpts::InlineTestDetailed => true,
        }
    }

    #[must_use]
    pub(crate) fn is_inline(&self) -> bool {
        match self {
            FormatOpts::YamlFormat => false,
            FormatOpts::InlineTest | FormatOpts::InlineTestDetailed => true,
        }
    }

    #[must_use]
    pub fn emit_stream(&self) -> bool {
        match self {
            FormatOpts::YamlFormat => true,
            FormatOpts::InlineTest | FormatOpts::InlineTestDetailed => false,
        }
    }
}

#[doc(hidden)]
pub fn write_str_from_event<T: Source>(
    line: &mut String,
    parser: &mut YamParser<T>,
    emit_opt: &FormatOpts,
) {
    if !emit_opt.emit_stream() {
        line.push('\n');
    }
    loop {
        let parsed = parser.next();
        if parsed.is_none() {
            break;
        }
        if let Some(Err(err)) = parsed {
            if emit_opt.emit_err() {
                let _ = writeln!(line, "ERR {err:?}");
            }
            break;
        }
        if let Some(Ok((ev, _))) = parsed {
            let _ = match ev {
                Event::StreamStart if emit_opt.emit_stream() => writeln!(line, "+STR"),
                Event::StreamEnd if emit_opt.emit_stream() => writeln!(line, "-STR"),
                Event::DocumentStart(_) => writeln!(line, "+DOC"),
                Event::DocumentEnd => writeln!(line, "-DOC"),
                Event::Alias(anchor_id) => {
                    let anchor = parser
                        .get_anchor(anchor_id)
                        .map_or(String::default(), ToString::to_string);
                    writeln!(line, "=ALI *{anchor}")
                }
                Event::Scalar(ScalarValue {
                    value,
                    scalar_type,
                    anchor_id,
                    tag,
                }) => {
                    let anchor = extract_anchor(parser, anchor_id);
                    let tag_info = extract_tag_full(tag);
                    writeln!(line, "=VAL{anchor}{tag_info} {scalar_type}{value}")
                }
                Event::SequenceStart(anchor_id, tag) => {
                    let tag_info = extract_tag_full(tag);
                    let anchor = extract_anchor(parser, anchor_id);
                    writeln!(line, "+SEQ{anchor}{tag_info}")
                }
                Event::SequenceEnd => writeln!(line, "-SEQ"),
                Event::MappingStart(anchor_id, tag) => {
                    let tag_info = extract_tag_full(tag);
                    let anchor = extract_anchor(parser, anchor_id);
                    writeln!(line, "+MAP{anchor}{tag_info}")
                }
                Event::MappingEnd => writeln!(line, "-MAP"),
                _ => write!(line, ""),
            };
        }
    }
    if emit_opt.is_inline() {
        *line = line.trim_end().into();
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

fn extract_anchor<T: Source>(parser: &YamParser<T>, anchor_id: usize) -> String {
    let anchor = if let Some(cow) = parser.get_anchor(anchor_id) {
        let mut out = String::with_capacity(cow.len() + 2);
        write!(out, " &{cow}").unwrap();
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

/// Test method for unescaping test cases.
///
/// # Panics
/// When unescaping the text it's expected to be
#[must_use]
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

    String::from_utf8(output).expect("Expected to escape valid UTF-8")
}
