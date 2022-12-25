use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::str::from_utf8_unchecked;
use crate::Scanner;
use crate::tokenizer::SpanToken::*;
use crate::tokenizer::StrReader;


pub struct EventIterator<'a> {
    pub(crate) state: Scanner,
    pub(crate) reader: StrReader<'a>,
    inner_cow: Cow<'a, [u8]>,
    indent: u32,
}

impl<'a> EventIterator<'a> {
    pub fn new_from_string(input: &str) -> EventIterator {
        EventIterator {
            state: Scanner::default(),
            reader: StrReader::new(input),
            inner_cow: Cow::default(),
            indent: 0,
        }
    }
}

impl<'a> EventIterator<'a> {
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

impl<'a> Iterator for EventIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut ind = vec![b'\n'];
            if let Some(token) = self.state.pop_token() {
                match token {
                    MarkStart(start) => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("=VAL ".as_bytes());
                        self.inner_cow = Cow::Owned(ind);
                        if let Some(MarkEnd(end)) = self.state.peek_token() {
                            self.state.pop_token();
                            self.merge_cow(start, end);
                        }
                    }
                    Space => self.merge_space(),
                    NewLine => self.merge_newline(),
                    MappingStart => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("+MAP".as_bytes());
                        self.indent += 2;
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    MappingEnd => {
                        self.indent -= 2;
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("-MAP".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    SequenceStart => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("+SEQ".as_bytes());
                        self.indent += 2;
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    SequenceEnd => {
                        self.indent -= 2;
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("-SEQ".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    DocumentStart => {
                        self.indent += 2;
                        ind.extend("+DOC".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    DocumentEnd => {
                        self.indent -= 2;
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("-DOC".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    KeyEnd => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("-KEY-".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    Directive(typ) => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend(format!("#{:} ", typ).as_bytes());
                        self.inner_cow.to_mut().extend(ind);
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
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend("-SEP-".as_bytes());
                        self.inner_cow.to_mut().extend(ind);
                        unsafe {
                            let x = mem::take(&mut self.inner_cow);
                            return Some(String::from_utf8_unchecked(x.to_vec()));
                        }
                    }
                    ErrorToken(x) => {
                        ind.extend(" ".repeat(self.indent as usize).as_bytes().to_vec());
                        ind.extend(format!("ERR({:?})",x).as_bytes());
                        self.inner_cow.to_mut().extend(ind);
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
