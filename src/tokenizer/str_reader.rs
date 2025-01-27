use std::collections::VecDeque;

use crate::tokenizer::SpanToken::*;
use crate::tokenizer::StrReader;
use crate::Spanner;

pub struct EventIterator<'a> {
    pub(crate) state: Spanner,
    pub(crate) reader: StrReader<'a>,
    indent: usize,
    pub(crate) lines: VecDeque<String>,
}

impl<'a> EventIterator<'a> {
    pub fn new_from_string(input: &str) -> EventIterator {
        EventIterator {
            state: Spanner::default(),
            reader: StrReader::new(input),
            indent: 2,
            lines: VecDeque::default(),
        }
    }
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.lines.is_empty() {
                return self.lines.pop_front();
            }

            if self.state.is_empty() && !self.state.stream_end {
                self.state.fetch_next_token(&mut self.reader);
                let mut start = 0;

                while let Some(token) = self.state.pop_token() {
                    let mut ind = vec![b'\n'];
                    match token {
                        MarkStart(index) => start = index,
                        MarkEnd(end) => {
                            let scalar = self.reader.slice[start..end].to_owned();

                            if let Some(x) = self.lines.back_mut() {
                                // account for indent and newline
                                if x[self.indent + 1..].starts_with("=VAL") {
                                    x.push_str(scalar.as_str());
                                };
                            } else {
                                ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                                ind.extend("=VAL ".as_bytes());
                                ind.extend(scalar.as_bytes().to_vec());
                                unsafe {
                                    self.lines.push_back(String::from_utf8_unchecked(ind));
                                }
                            }
                        }
                        NewLine(n) => {
                            if let Some(x) = self.lines.back_mut() {
                                for _ in 0..n {
                                    x.push('\n');
                                }
                            }
                        }
                        Space => {
                            if let Some(x) = self.lines.back_mut() {
                                x.push(' ');
                            }
                        }
                        Alias => {}
                        MappingStart => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("+MAP".as_bytes());
                            self.indent += 2;
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        MappingEnd => {
                            self.indent -= 2;
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("-MAP".as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        SequenceStart => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("+SEQ".as_bytes());
                            self.indent += 2;
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        SequenceEnd => {
                            self.indent -= 2;
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("-SEQ".as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        DocumentStart => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("+DOC".as_bytes());
                            self.indent += 2;
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        DocumentEnd => {
                            self.indent -= 2;
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("-DOC".as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        KeyEnd => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("-KEY-".as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        Directive(typ) => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend(format!("#{:} ", typ).as_bytes());
                            if let (Some(MarkStart(start)), Some(MarkEnd(end))) =
                                (self.state.pop_token(), self.state.pop_token())
                            {
                                ind.extend(self.reader.slice[start..end].as_bytes());
                            }
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        Separator => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend("-SEP-".as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                        ErrorToken(x) => {
                            ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                            ind.extend(format!("ERR({:?})", x).as_bytes());
                            unsafe {
                                self.lines.push_back(String::from_utf8_unchecked(ind));
                            }
                        }
                    };
                }
            }
            if self.state.is_empty() && self.state.stream_end && self.lines.is_empty() {
                return None;
            }
        }
    }
}
