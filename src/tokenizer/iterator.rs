use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::path::Path;
use std::{fmt::Write, io, str::from_utf8_unchecked};

use crate::escaper::escape_plain;
use crate::tokenizer::iterator::Event::ErrorEvent;
use crate::tokenizer::{Reader, Slicer};
use crate::Lexer;

use super::StrReader;

///
/// Iterator over events
///
/// It returns borrowed events that correspond to the
/// It's generic over:
/// `'a` - lifetime
/// [R] - Reader
/// [B] - Buffer Type
/// [S] - Input source
pub struct EventIterator<'a, R, B = (), S = &'a mut [u8]> {
    /// Reader type that usually implements a [Reader] trait which takes a Buffer type [B]
    pub(crate) reader: R,
    /// Lexer which controls current state of parsing
    pub(crate) state: Lexer,
    /// Current event indentation level
    pub indent: usize,
    /// Helper to store the unconstrained types
    phantom: PhantomData<(&'a B, S)>,
}

impl<'a, R, B, S> EventIterator<'a, R, B, S> {
    #[inline]
    pub fn new(reader: R) -> EventIterator<'a, R, B, S> {
        EventIterator {
            reader,
            state: Lexer::default(),
            indent: 1,
            phantom: PhantomData::default(),
        }
    }
}

impl<'a, R, B> From<&'a str> for EventIterator<'a, R, B>
where
    R: Reader<B> + From<&'a str>,
{
    fn from(value: &'a str) -> Self {
        EventIterator::new(From::from(value))
    }
}

impl<'a, R, B> From<&'a [u8]> for EventIterator<'a, R, B>
where
    R: Reader<B> + From<&'a [u8]>,
{
    fn from(value: &'a [u8]) -> Self {
        EventIterator::new(From::from(value))
    }
}

impl<'a, R, B, S: BufRead> EventIterator<'a, R, B, S>
where
    R: Reader<B> + From<S>,
{
    pub fn from_buf(value: S) -> Self {
        EventIterator::new(From::from(value))
    }
}

impl<'a, R, B> EventIterator<'a, R, B, BufReader<File>>
where
    R: Reader<B> + From<BufReader<File>>,
{
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(EventIterator::new(From::from(reader)))
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScalarType {
    Plain,
    Folded,
    Literal,
    SingleQuote,
    DoubleQuote,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

#[derive(Clone, PartialEq)]
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
    ErrorEvent,
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
            ErrorEvent => {
                write!(f, "ERR")
            }
            _ => Ok(()),
            // Event::Tag(_) => todo!(),
            // Event::Alias(_) => todo!(),
            // Event::Anchor(_) => todo!(),
        }
    }
}

impl<'a, R, B> Iterator for EventIterator<'a, R, B>
where
    R: Slicer<'a> + Reader<B>,
{
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
                        return Some((
                            DocStart {
                                explicit: self.state.directive,
                            },
                            curr_indent,
                        ));
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
                    ErrorToken => return Some((ErrorEvent, curr_indent)),
                    DirectiveReserved | DirectiveTag | DirectiveYaml => {
                        let directive_type = unsafe { token.to_yaml_directive() };
                        return if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            let slice = Cow::Borrowed(self.reader.slice(start, end));
                            Some((
                                Directive {
                                    directive_type,
                                    value: slice,
                                },
                                curr_indent,
                            ))
                        } else {
                            panic!("Error in processing YAML file");
                        };
                    }
                    ScalarPlain | ScalarLit | ScalarFold | ScalarDoubleQuote
                    | ScalarSingleQuote | Mark => {
                        // Safe if only one of these six
                        let scalar_type = unsafe { token.to_scalar() };
                        let mut cow: Cow<'a, [u8]> = Cow::default();
                        loop {
                            match (self.state.peek_token(), self.state.peek_token_next()) {
                                (Some(start), Some(end))
                                    if start < NewLine as usize && end < NewLine as usize =>
                                {
                                    if cow.is_empty() {
                                        cow = Cow::Borrowed(self.reader.slice(start, end));
                                    } else {
                                        cow.to_mut().extend(self.reader.slice(start, end))
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
                        let cow = match scalar_type {
                            ScalarType::Plain => escape_plain(cow),
                            _ => cow,
                        };
                        return Some((
                            Scalar {
                                scalar_type,
                                value: cow,
                            },
                            curr_indent,
                        ));
                    }
                    AliasToken => todo!(),
                    AnchorToken => todo!(),
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
    let scan: EventIterator<'_, StrReader> = EventIterator::from(input_yaml);
    scan.for_each(|(ev, indent)| {
        line.push('\n');
        line.push_str(&" ".repeat(indent));
        write!(line, "{:}", ev).unwrap();
    });

    assert_eq!(expect, line, "Error in {input_yaml}");
}
