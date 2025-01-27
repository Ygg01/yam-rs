use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::str::from_utf8_unchecked;

use steel_yaml::Scanner;
use steel_yaml::tokenizer::{DirectiveType, ErrorType, Reader, SpanToken, StrReader};
use steel_yaml::tokenizer::SpanToken::*;

pub struct StrIterator<'a> {
    pub(crate) state: Scanner,
    pub(crate) reader: StrReader<'a>,
    inner_cow: Cow<'a, [u8]>,
    indent: u32,
}

impl<'a> StrIterator<'a> {
    pub fn new_from_string(input: &str) -> StrIterator {
        StrIterator {
            state: Scanner::default(),
            reader: StrReader::new(input),
            inner_cow: Cow::default(),
            indent: 0,
        }
    }
}

impl<'a> StrIterator<'a> {
    fn to_cow(&self, start: usize, end: usize) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.reader.slice[start..end].as_bytes())
    }

    fn merge_cow(&mut self, start: usize, end: usize) {
        if self.inner_cow.is_empty() {
            self.inner_cow = Cow::Borrowed(self.reader.slice[start..end].as_bytes());
        } else {
            self.inner_cow
                .to_mut()
                .extend(self.reader.slice[start..end].as_bytes());
        }
    }

    fn merge_space(&mut self) {
        self.inner_cow.to_mut().push(b' ');
    }

    fn merge_newline(&mut self) {
        self.inner_cow.to_mut().push(b'\n');
    }
}

impl<'a> Iterator for StrIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(token) = self.state.pop_token() {
                match token {
                    MarkStart(start) => {
                        self.inner_cow = Cow::Owned("\n=VAL ".as_bytes().to_vec());
                        if let Some(MarkEnd(end)) = self.state.peek_token() {
                            self.state.pop_token();
                            self.merge_cow(start, end);
                        }
                    }
                    Space => self.merge_space(),
                    NewLine => self.merge_newline(),
                    MappingStart => {
                        self.inner_cow.to_mut().extend("\n+MAP".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    MappingEnd => {
                        self.inner_cow.to_mut().extend("\n-MAP".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    SequenceStart => {
                        self.inner_cow.to_mut().extend("\n+SEQ".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    SequenceEnd => {
                        self.inner_cow.to_mut().extend("\n-SEQ".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    DocumentStart => {
                        self.inner_cow.to_mut().extend("\n+DOC".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    DocumentEnd => {
                        self.inner_cow.to_mut().extend("\n-DOC".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    KeyEnd => {
                        self.inner_cow.to_mut().extend("\n-KEY-".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    Directive(typ) => {
                        self.inner_cow
                            .to_mut()
                            .extend(format!("\n#{:} ", typ).as_bytes());
                        if let (Some(MarkStart(start)), Some(MarkEnd(end))) =
                            (self.state.pop_token(), self.state.pop_token())
                        {
                            self.merge_cow(start, end);
                        }
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    Separator => {
                        self.inner_cow.to_mut().extend("\n-SEP-".as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    ErrorToken(x) => {
                        let s = format!("\nERR({:?})", x);
                        self.inner_cow.to_mut().extend(s.as_bytes());
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    _ => {}
                }
            } else {
                // Deal with any leftover events
                if !self.inner_cow.is_empty() {
                    unsafe {
                        let x = mem::take(&mut self.inner_cow);
                        return Some(String::from_utf8_unchecked(x.to_vec()));
                    }
                }
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
