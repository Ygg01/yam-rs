use std::borrow::Cow;

use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::StrReader;
use crate::tokenizer::scanner::SpanToken;
use crate::Scanner;

pub struct StrIterator<'a> {
    pub(crate) state: Scanner,
    pub(crate) reader: StrReader<'a>,
}

impl<'a> StrIterator<'a> {
    pub(crate) fn to_token(&self, token: SpanToken) -> YamlEvent<'a> {
        match token {
            SpanToken::DocStart => YamlEvent::DocStart,
            SpanToken::DocEnd => YamlEvent::DocEnd,
            SpanToken::StreamStart => YamlEvent::StreamStart,
            SpanToken::StreamEnd => YamlEvent::StreamEnd,
            SpanToken::Scalar(start, end) => YamlEvent::ScalarValue(self.to_cow(start, end)),
            SpanToken::Directive(typ, start, end) => {
                YamlEvent::Directive(typ, self.to_cow(start, end))
            }
            SpanToken::ErrorToken(err) => YamlEvent::Error(err),
            _ => YamlEvent::StreamStart,
        }
    }

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
        Some(self.to_token(span))
    }
}

#[derive(Copy, Clone)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedSymbol,
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedIndent(usize),
    StartedBlockInFlow,
}
