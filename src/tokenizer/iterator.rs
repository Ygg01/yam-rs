use std::borrow::Cow;
use std::fmt::Display;
use std::{fmt::Write, str::from_utf8_unchecked};

use crate::{tokenizer::LexerToken, Spanner};

use super::StrReader;

pub struct EventIterator<'a> {
    pub(crate) reader: StrReader<'a>,
    pub(crate) state: Spanner,
    pub indent: usize,
}

impl<'a> EventIterator<'a> {
    pub fn new_from_string(input: &str) -> EventIterator {
        EventIterator {
            reader: StrReader::new(input),
            state: Spanner::default(),
            indent: 1,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ScalarType {
    Plain,
    Folded,
    Literal,
    SingleQuote,
    DoubleQuote,
}

#[derive(Copy, Clone)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

#[derive(Clone)]
pub enum Event<'a> {
    DocStart {
        explicit: bool,
    },
    DocEnd,
    SeqStart {
        flow: bool,
    },
    SeqEnd,
    MapStart {
        flow: bool,
    },
    MapEnd,
    Directive {
        directive_type: DirectiveType,
        value: Cow<'a, [u8]>,
    },
    Scalar {
        scalar_type: ScalarType,
        value: Cow<'a, [u8]>,
    },
    Tag(Cow<'a, [u8]>),
    Alias(Cow<'a, [u8]>),
    Anchor(Cow<'a, [u8]>),
    Error,
}

impl<'a> Display for Event<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::DocStart { explicit } => {
                let exp_str = if *explicit { " ---" } else { "" };
                write!(f, "+DOC{}", exp_str)
            }
            Event::DocEnd => {
                write!(f, "-DOC")
            }
            Event::SeqStart { flow } => {
                let flow_str = if *flow { " []" } else { "" };
                write!(f, "+SEQ{}", flow_str)
            }
            Event::SeqEnd => {
                write!(f, "-SEQ")
            }
            Event::MapStart { flow } => {
                let flow_str = if *flow { " {}" } else { "" };
                write!(f, "+MAP{}", flow_str)
            }
            Event::MapEnd => {
                write!(f, "-MAP")
            }
            Event::Directive {
                directive_type,
                value,
            } => {
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                match directive_type {
                    DirectiveType::Yaml => write!(f, "%YAML {}", val_str),
                    _ => write!(f, "{}", val_str),
                }
            }
            Event::Scalar { scalar_type, value } => {
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                write!(f, "=VAL ")?;
                match *scalar_type {
                    ScalarType::Plain => write!(f, ":"),
                    ScalarType::Folded => write!(f, ">"),
                    ScalarType::Literal => write!(f, "|"),
                    ScalarType::SingleQuote => write!(f, "\'"),
                    ScalarType::DoubleQuote => write!(f, "\""),
                }?;
                write!(f, "{}", val_str)
            }
            Event::Error => {
                write!(f, "ERR")
            }
            _ => Ok(()),
            // Event::Tag(_) => todo!(),
            // Event::Alias(_) => todo!(),
            // Event::Anchor(_) => todo!(),
        }
    }
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = (Event<'a>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        pub use crate::tokenizer::iterator::Event::*;
        pub use crate::tokenizer::LexerToken::*;

        loop {
            if self.state.is_empty() && !self.state.stream_end {
                self.state.fetch_next_token(&mut self.reader);
            }

            let curr_indent = self.indent;
            if let Some(x) = self.state.pop_token() {
                let token = x.into();
                match token {
                    SequenceStart => {
                        self.indent += 1;
                        return Some((
                            SeqStart {
                                flow: self.state.curr_state.in_flow_collection(),
                            },
                            curr_indent,
                        ));
                    }
                    MappingStart => {
                        self.indent += 1;
                        return Some((
                            MapStart {
                                flow: self.state.curr_state.in_flow_collection(),
                            },
                            curr_indent,
                        ));
                    }
                    DocumentStart => {
                        self.indent += 1;
                        return Some((DocStart { explicit: true }, curr_indent));
                    }
                    SequenceEnd => {
                        self.indent -= 1;
                        return Some((SeqEnd, self.indent));
                    }
                    MappingEnd => {
                        self.indent -= 1;
                        return Some((MapEnd, self.indent));
                    }
                    DocumentEnd => {
                        self.indent -= 1;
                        return Some((DocEnd, self.indent));
                    }
                    LexerToken::Error => return Some((Event::Error, curr_indent)),
                    DirectiveReserved | DirectiveTag | DirectiveYaml => {
                        let directive_type = unsafe { token.to_yaml_directive() };
                        return if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            let slice = Cow::Borrowed(&self.reader.slice[start..end]);
                            Some((
                                Directive {
                                    directive_type,
                                    value: slice,
                                },
                                curr_indent,
                            ))
                        } else {
                            panic!("Error in proccessing YAML file");
                        };
                    }
                    ScalarPlain | ScalarLit | ScalarFold | ScalarDoubleQuote
                    | ScalarSingleQuote | Mark => {
                        // Safe if only one of these five
                        let scalar_type = unsafe { token.to_scalar() };
                        let mut cow: Cow<'a, [u8]> = Cow::default();
                        loop {
                            match (self.state.peek_token(), self.state.peek_token_next()) {
                                (Some(start), Some(end))
                                    if start < NewLine as usize && end < NewLine as usize =>
                                {
                                    if cow.is_empty() {
                                        cow = Cow::Borrowed(&self.reader.slice[start..end]);
                                    } else {
                                        cow.to_mut().extend(&self.reader.slice[start..end])
                                    }
                                    self.state.pop_token();
                                    self.state.pop_token();
                                }
                                (Some(newline), Some(line)) if newline == NewLine as usize => {
                                    if line == 0 {
                                        cow.to_mut().extend(" ".as_bytes());
                                    } else {
                                        cow.to_mut().extend("\\n".repeat(line).as_bytes())
                                    }
                                    self.state.pop_token();
                                    self.state.pop_token();
                                }
                                (_, _) => {
                                    break;
                                }
                            }
                        }
                        return Some((
                            Event::Scalar {
                                scalar_type,
                                value: cow,
                            },
                            curr_indent,
                        ));
                    }
                    LexerToken::Alias => todo!(),
                    LexerToken::Anchor => todo!(),
                    TagStart => todo!(),
                    NewLine | ScalarEnd => {}
                }
            }
            if self.state.stream_end && self.state.is_empty() {
                return None;
            }
        }
    }
}

pub fn assert_eq_event(input_yaml: &str, expect: &str) {
    let mut line = String::new();
    let scan = EventIterator::new_from_string(input_yaml);
    scan.for_each(|(ev, indent)| {
        line.push('\n');
        line.push_str(&" ".repeat(indent));
        write!(line, "{:}", ev);
    });

    assert_eq!(expect, line, "Error in {input_yaml}");
}
