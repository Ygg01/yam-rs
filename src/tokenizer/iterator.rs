use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;

use std::{fmt::Write, io, str::from_utf8_unchecked};

use urlencoding::decode_binary;

use crate::escaper::{escape_double_quotes, escape_plain};
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
/// [RB] - Reader Buffer
/// [I] - Input Buffer (optional)
pub struct EventIterator<'a, R, RB = &'a [u8], I = ()> {
    /// Reader type that usually implements a [Reader] trait which takes a Buffer type [B]
    pub(crate) reader: R,
    pub(crate) buffer: RB,
    /// Lexer which controls current state of parsing
    pub(crate) state: Lexer<I>,
    /// Tag of current node,
    pub(crate) tag: Option<Cow<'a, [u8]>>,
    /// Alias of current node,
    pub(crate) anchor: Option<Cow<'a, [u8]>>,
    /// Helper to store the unconstrained types
    phantom: PhantomData<(&'a I, RB)>,
}

impl<'a> From<&'a str> for EventIterator<'a, StrReader<'a>, &'a [u8]> {
    fn from(value: &'a str) -> Self {
        EventIterator {
            reader: StrReader::from(value),
            state: Lexer::new_from_buf(()),
            buffer: value.as_bytes(),
            tag: None,
            anchor: None,
            phantom: PhantomData::default(),
        }
    }
}

impl<'a> From<&'a [u8]> for EventIterator<'a, StrReader<'a>, &'a [u8]> {
    fn from(value: &'a [u8]) -> Self {
        EventIterator {
            reader: StrReader::from(value),
            state: Lexer::new_from_buf(()),
            buffer: value,
            tag: None,
            anchor: None,
            phantom: PhantomData::default(),
        }
    }
}

impl<'a, R, RB, B> EventIterator<'a, R, RB, B> {
    #[inline]
    pub fn new(reader: R, buffer: RB, src: B) -> EventIterator<'a, R, RB, B> {
        EventIterator {
            reader,
            buffer,
            state: Lexer::new_from_buf(src),
            tag: None,
            anchor: None,
            phantom: PhantomData::default(),
        }
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
    DocEnd {
        explicit: bool,
    },
    SeqStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        flow: bool,
    },
    SeqEnd,
    MapStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        flow: bool,
    },
    MapEnd,
    Directive {
        directive_type: DirectiveType,
        value: Cow<'a, [u8]>,
    },
    Scalar {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        scalar_type: ScalarType,
        value: Cow<'a, [u8]>,
    },
    Alias(Cow<'a, [u8]>),
    ErrorEvent,
}

impl<'a> Display for Event<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::DocStart { explicit } => {
                let exp_str = if *explicit { " ---" } else { "" };
                write!(f, "+DOC{}", exp_str)
            }
            Event::DocEnd { explicit } => {
                let exp_str = if *explicit { " ..." } else { "" };
                write!(f, "-DOC{}", exp_str)
            }
            Event::SeqStart { flow, tag, anchor } => {
                write!(f, "+SEQ",)?;
                if *flow {
                    write!(f, " []")?;
                }
                if let Some(cow) = anchor {
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{}", string)?;
                };
                if let Some(cow) = tag {
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{}>", string)?;
                };
                Ok(())
            }
            Event::SeqEnd => {
                write!(f, "-SEQ")
            }
            Event::MapStart { flow, tag, anchor } => {
                write!(f, "+MAP")?;
                if *flow {
                    write!(f, " {{}}")?;
                }
                if let Some(cow) = anchor {
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{}", string)?;
                };
                if let Some(cow) = tag {
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{}>", string)?;
                };
                Ok(())
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
            Event::Scalar {
                scalar_type,
                value,
                tag,
                anchor,
            } => {
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                write!(f, "=VAL")?;

                if let Some(cow) = anchor {
                    let string: &str = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{}", string)?;
                };
                if let Some(cow) = tag {
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{}>", string)?;
                };
                match *scalar_type {
                    ScalarType::Plain => write!(f, " :"),
                    ScalarType::Folded => write!(f, " >"),
                    ScalarType::Literal => write!(f, " |"),
                    ScalarType::SingleQuote => write!(f, " \'"),
                    ScalarType::DoubleQuote => write!(f, " \""),
                }?;
                write!(f, "{}", val_str)?;

                Ok(())
            }
            ErrorEvent => {
                write!(f, "ERR")
            }
            Event::Alias(value) => {
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                write!(f, "=ALI *{}", val_str)
            }
        }
    }
}

impl<'a> Slicer<'a> for &'a [u8] {
    fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        unsafe { self.get_unchecked(start..end) }
    }
}

impl<'a, R, RB, B> Iterator for EventIterator<'a, R, RB, B>
where
    R: Reader<B>,
    RB: Slicer<'a>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        pub use crate::tokenizer::iterator::Event::*;
        pub use crate::tokenizer::LexerToken::*;

        loop {
            if self.state.is_empty() && !self.state.stream_end {
                self.state.fetch_next_token(&mut self.reader);
            }

            if let Some(x) = self.state.pop_token() {
                let token = x.into();
                let tag = self.tag.take();
                let anchor = self.anchor.take();
                match token {
                    SequenceStart => {
                        return Some(SeqStart {
                            flow: true,
                            tag,
                            anchor,
                        });
                    }
                    SequenceStartImplicit => {
                        return Some(SeqStart {
                            flow: false,
                            tag,
                            anchor,
                        });
                    }
                    MappingStart => {
                        return Some(MapStart {
                            flow: true,
                            tag,
                            anchor,
                        });
                    }
                    MappingStartImplicit => {
                        return Some(MapStart {
                            flow: false,
                            tag,
                            anchor,
                        });
                    }
                    DocumentStart => {
                        return Some(DocStart { explicit: false });
                    }
                    DocumentStartExplicit => {
                        return Some(DocStart { explicit: true });
                    }
                    SequenceEnd => {
                        return Some(SeqEnd);
                    }
                    MappingEnd => {
                        return Some(MapEnd);
                    }
                    DocumentEnd => {
                        return Some(DocEnd { explicit: false });
                    }
                    DocumentEndExplicit => {
                        return Some(DocEnd { explicit: true });
                    }
                    ErrorToken => return Some(ErrorEvent),
                    DirectiveReserved | DirectiveTag | DirectiveYaml => {
                        let directive_type = unsafe { token.to_yaml_directive() };
                        return if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            let slice = Cow::Borrowed(self.buffer.slice(start, end));
                            Some(
                                Directive {
                                    directive_type,
                                    value: slice,
                                },
                        )
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
                                        cow = Cow::Borrowed(self.buffer.slice(start, end));
                                    } else {
                                        cow.to_mut().extend(self.buffer.slice(start, end))
                                    }
                                    self.state.pop_token();
                                    self.state.pop_token();
                                }
                                (Some(newline), Some(line)) if newline == NewLine as usize => {
                                    if line == 0 {
                                        cow.to_mut().extend(" ".as_bytes());
                                    } else {
                                        cow.to_mut().extend("\n".repeat(line).as_bytes())
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
                            ScalarType::Plain | ScalarType::Literal | ScalarType::Folded => {
                                escape_plain(cow)
                            }
                            ScalarType::DoubleQuote => escape_double_quotes(cow),
                            _ => cow,
                        };
                        return Some(
                            Scalar {
                                scalar_type,
                                value: cow,
                                tag,
                                anchor,
                            }
                        );
                    }
                    AliasToken => {
                        if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            return Some(
                                Alias(Cow::Borrowed(self.buffer.slice(start, end))),
                            );
                        }
                    }
                    AnchorToken => {
                        if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            self.anchor = Some(Cow::Borrowed(self.buffer.slice(start, end)));
                        }
                    }
                    TagStart => {
                        if let (Some(start), Some(mid), Some(end)) = (
                            self.state.pop_token(),
                            self.state.pop_token(),
                            self.state.pop_token(),
                        ) {
                            let namespace = self.buffer.slice(start, mid);
                            let extension = self.buffer.slice(mid, end);
                            self.tag = if let Some(&(e1, e2)) = self.state.tags.get(namespace) {
                                let mut tag = Vec::from(self.buffer.slice(e1, e2));
                                tag.extend_from_slice(extension);
                                if tag.contains(&b'%') {
                                    tag = decode_binary(&tag).into_owned()
                                }
                                Some(Cow::Owned(tag))
                            } else if namespace == b"!!" && !extension.is_empty() {
                                let mut cow: Cow<'_, [u8]> =
                                    Cow::Owned(b"tag:yaml.org,2002:".to_vec());
                                cow.to_mut().extend(extension);
                                Some(cow)
                            } else if namespace == b"!" {
                                let mut cow: Cow<'_, [u8]> = Cow::Owned(b"!".to_vec());
                                cow.to_mut().extend(extension);
                                Some(cow)
                            } else {
                                None
                            }
                        }
                    }
                    NewLine | ScalarEnd => {}
                }
            }
            if self.state.stream_end && self.state.is_empty() {
                return None;
            }
        }
    }
}

pub fn assert_eq_event(input: &str, events: &str) {
    let mut line = String::new();
    let scan: EventIterator<'_, StrReader, _> = EventIterator::from(input);
    scan.for_each(|ev| {
        line.push('\n');
        write!(line, "{:}", ev).unwrap();
    });

    assert_eq!(events, line, "Error in {input}");
}
