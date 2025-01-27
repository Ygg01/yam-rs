use std::borrow::Cow;

use SpanToken::*;

use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::StrReader;
use crate::tokenizer::scanner::SpanToken;
use crate::tokenizer::scanner::SpanToken::{MarkEnd, MarkStart};
use crate::Scanner;

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
        let span = loop {
            if let Some(token) = self.state.pop_token() {
                break token;
            } else {
                if self.state.is_empty() && !self.state.stream_end {
                    self.state.fetch_next_token(&mut self.reader);
                }
                if self.state.is_empty() && self.state.stream_end {
                    return None;
                }
            }
        };
        let event = match span {
            Directive(tag) => {
                if let (Some(MarkStart(start)), Some(MarkEnd(end))) =
                    (self.state.pop_token(), self.state.pop_token())
                {
                    YamlEvent::Directive(tag, self.to_cow(start, end))
                } else {
                    YamlEvent::Error(ErrorType::UnexpectedEndOfFile)
                }
            }
            MarkStart(start) => {
                if let Some(MarkEnd(end)) = self.state.pop_token() {
                    YamlEvent::ScalarValue(self.to_cow(start, end))
                } else {
                    YamlEvent::Error(ErrorType::UnexpectedEndOfFile)
                }
            }
            MarkEnd(_) => panic!("Unexpected Mark end"),
            DocumentStart => YamlEvent::DocStart,
            DocumentEnd => YamlEvent::DocEnd,
            SequenceStart => YamlEvent::SeqStart,
            SequenceEnd => YamlEvent::SeqEnd,
            MappingStart => YamlEvent::MapStart,
            MappingEnd => YamlEvent::MapEnd,
            ErrorToken(err) => YamlEvent::Error(err),
        };
        Some(event)
    }
}

#[derive(Copy, Clone)]
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
