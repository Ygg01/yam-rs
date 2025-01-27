use std::borrow::Cow;

use SpanToken::*;

use crate::Scanner;
use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::StrReader;
use crate::tokenizer::scanner::SpanToken;
use crate::tokenizer::scanner::SpanToken::{MarkEnd, MarkStart};

pub struct StrIterator<'a> {
    pub(crate) state: Scanner,
    pub(crate) reader: StrReader<'a>,
}

impl<'a> StrIterator<'a> {
    fn to_cow(&self, start: usize, end: usize) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.reader.slice[start..end].as_bytes())
    }
}

impl<'a> Iterator for StrIterator<'a> {
    type Item = YamlEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(token) = self.state.pop_token() {
                match token {
                    Directive(tag) => {
                        if let (Some(MarkStart(start)), Some(MarkEnd(end))) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            return Some(YamlEvent::Directive(tag, self.to_cow(start, end)));
                        }
                    }
                    MarkStart(start) => {
                        let mut borrow = Cow::default();

                        if let Some(MarkEnd(end)) = self.state.pop_token() {
                            borrow = self.to_cow(start, end);
                        }
                        loop {
                            match self.state.peek_token() {
                                Some(MarkStart(x0)) => {
                                    self.state.pop_token();
                                    if let Some(MarkEnd(x1)) = self.state.pop_token() {
                                        borrow.to_mut().extend(self.to_cow(x0, x1).to_vec());
                                    }
                                }
                                Some(NewLine) => {
                                    self.state.pop_token();
                                    borrow.to_mut().push(b'\n');
                                }
                                Some(Space) => {
                                    self.state.pop_token();
                                    borrow.to_mut().push(b' ');
                                }
                                _ => break,
                            };
                        }
                        return Some(YamlEvent::ScalarValue(borrow));
                    }

                    MarkEnd(_) => panic!("Unexpected Mark end"),
                    DocumentStart => return Some(YamlEvent::DocStart),
                    DocumentEnd => return Some(YamlEvent::DocEnd),
                    SequenceStart => return Some(YamlEvent::SeqStart),
                    SequenceEnd => return Some(YamlEvent::SeqEnd),
                    MappingStart => return Some(YamlEvent::MapStart),
                    MappingEnd => return Some(YamlEvent::MapEnd),
                    ErrorToken(err) => return Some(YamlEvent::Error(err)),
                    _ => {}
                };
            } else {
                if self.state.is_empty() && !self.state.stream_end {
                    self.state.fetch_next_token(&mut self.reader);
                }
                if self.state.is_empty() && self.state.stream_end {
                    return None;
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    ExpectedIndent(u32),
    StartedBlockInFlow,
}
