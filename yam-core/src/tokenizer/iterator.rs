use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::Lexer;
use crate::escaper::{escape_double_quotes, escape_plain, escape_single_quotes};
use crate::tokenizer::{Reader, Slicer};
use urlencoding::decode_binary;
use yam_common::{ScalarType, YEvent};

use super::StrReader;

/// Iterator over events
///
/// It returns borrowed events that correspond to the YAML parsing events.
pub struct EventIterator<'a, R, RB = &'a [u8], I = ()> {
    /// Reader type that usually implements a [Reader] trait which takes a Buffer type [B]
    pub(crate) reader: R,
    /// Reader buffer, which is the internal buffer of the reader
    /// used to store data from intermittent sources like buffered readers.
    pub(crate) buffer: RB,
    /// Lexer, which controls the current state of parsing
    pub(crate) state: Lexer,
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
            state: Lexer::default(),
            buffer: value.as_bytes(),
            tag: None,
            anchor: None,
            phantom: PhantomData,
        }
    }
}

impl<'a> From<&'a [u8]> for EventIterator<'a, StrReader<'a>, &'a [u8]> {
    fn from(value: &'a [u8]) -> Self {
        EventIterator {
            reader: StrReader::from(value),
            state: Lexer::default(),
            buffer: value,
            tag: None,
            anchor: None,
            phantom: PhantomData,
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
    type Item = YEvent<'a>;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        pub use crate::tokenizer::LexerToken::*;
        pub use crate::tokenizer::iterator::YEvent::*;

        loop {
            if self.state.is_empty() && !self.state.stream_end {
                self.state.fetch_next_token(&mut self.reader);
            }

            if let Some(x) = self.state.pop_token() {
                let token = x.into();
                match token {
                    SequenceStart => {
                        return Some(SeqStart {
                            tag: self.tag.take(),
                            anchor: self.anchor.take(),
                        });
                    }
                    SequenceStartImplicit => {
                        return Some(SeqStart {
                            tag: self.tag.take(),
                            anchor: self.anchor.take(),
                        });
                    }
                    MappingStart => {
                        return Some(MapStart {
                            tag: self.tag.take(),
                            anchor: self.anchor.take(),
                        });
                    }
                    MappingStartImplicit => {
                        return Some(MapStart {
                            tag: self.tag.take(),
                            anchor: self.anchor.take(),
                        });
                    }
                    DocumentStart => {
                        return Some(DocStart);
                    }
                    DocumentStartExplicit => {
                        return Some(DocStart);
                    }
                    SequenceEnd => {
                        return Some(SeqEnd);
                    }
                    MappingEnd => {
                        return Some(MapEnd);
                    }
                    DocumentEnd => {
                        return Some(DocEnd);
                    }
                    DocumentEndExplicit => {
                        return Some(DocEnd);
                    }
                    ErrorToken => return Some(ErrorEvent),
                    DirectiveReserved | DirectiveTag | DirectiveYaml => {
                        let directive_type = unsafe { token.to_yaml_directive() };
                        return if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            let slice = Cow::Borrowed(self.buffer.slice(start, end));
                            Some(Directive {
                                directive_type,
                                value: slice,
                            })
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
                                        cow.to_mut().extend(self.buffer.slice(start, end));
                                    }
                                    self.state.pop_token();
                                    self.state.pop_token();
                                }
                                (Some(newline), Some(line)) if newline == NewLine as usize => {
                                    if line == 0 {
                                        cow.to_mut().extend(" ".as_bytes());
                                    } else {
                                        cow.to_mut().extend("\n".repeat(line).as_bytes());
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
                            ScalarType::SingleQuote => escape_single_quotes(cow),
                        };
                        return Some(Scalar {
                            scalar_type,
                            value: cow,
                            tag: self.tag.take(),
                            anchor: self.anchor.take(),
                        });
                    }
                    AliasToken => {
                        if let (Some(start), Some(end)) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            return Some(Alias(Cow::Borrowed(self.buffer.slice(start, end))));
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
                            let extension = if end == 0 {
                                b""
                            } else {
                                self.buffer.slice(mid, end)
                            };
                            self.tag = if let Some(&(e1, e2)) = self.state.tags.get(namespace) {
                                let mut tag = Vec::from(self.buffer.slice(e1, e2));
                                tag.extend_from_slice(extension);
                                if tag.contains(&b'%') {
                                    tag = decode_binary(&tag).into_owned();
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
                            } else if extension.is_empty() && end == 0 {
                                Some(Cow::Borrowed(namespace))
                            } else {
                                return Some(YEvent::ErrorEvent);
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

///
/// Assert that in for given input, the parser generates expected set of events
///
/// # Panics
///
///    Function panics if there is a difference between expected events string and one generated
///    from the input.
pub fn assert_eq_event(input: &str, expected_events: &str) {
    use core::fmt::Write;

    let mut line = String::with_capacity(expected_events.len());
    let scan: EventIterator<'_, StrReader, _> = EventIterator::from(input);
    scan.for_each(|ev| {
        line.push('\n');
        write!(line, "{ev:}").unwrap();
    });

    assert_eq!(line, expected_events, "Error in {input}");
}
