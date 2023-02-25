use std::collections::VecDeque;

use crate::Spanner;
use crate::tokenizer::SpanToken::*;
use crate::tokenizer::str_reader::StrReader;

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

                            match self.lines.back_mut() {
                                Some(line) if line[self.indent + 1..].starts_with("=VAL") => unsafe {
                                    line.as_mut_vec().extend(scalar);
                                },
                                _ => {
                                    ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                                    ind.extend("=VAL ".as_bytes());
                                    ind.extend(scalar.to_vec());
                                    unsafe {
                                        self.lines.push_back(String::from_utf8_unchecked(ind));
                                    }
                                }
                            };
                        }
                        NewLine(n) => {
                            if let Some(x) = self.lines.back_mut() {
                                for _ in 0..n {
                                    x.push_str("\\n");
                                }
                            }
                        }
                        Space => {
                            if let Some(x) = self.lines.back_mut() {
                                x.push(' ');
                            }
                        }
                        Alias | Anchor => {
                            if let (Some(MarkStart(start)), Some(MarkEnd(end))) =
                                    (self.state.pop_token(), self.state.pop_token())
                            {
                                let scalar = self.reader.slice[start..end].to_owned();
                                ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                                let is_alias = matches!(token, Alias);
                                if is_alias {
                                    ind.extend("ALIAS".as_bytes())
                                } else {
                                    ind.extend("ANCHOR".as_bytes())
                                }
                                ind.extend(scalar);
                                unsafe {
                                    self.lines.push_back(String::from_utf8_unchecked(ind))
                                }
                            }
                        }
                        TagStart(start) => {
                            if let (Some(MarkStart(mid)), Some(MarkEnd(end))) =
                                    (self.state.pop_token(), self.state.pop_token())
                            {
                                let tag_schema = self.reader.slice[start..mid].to_owned();
                                let tag = self.reader.slice[mid + 1..end].to_owned();
                                ind.extend(" ".repeat(self.indent).as_bytes().to_vec());
                                ind.push(b'!');
                                ind.extend(tag_schema);
                                ind.push(b'!');
                                ind.extend(tag);
                                unsafe {
                                    self.lines.push_back(String::from_utf8_unchecked(ind))
                                }
                            }
                        }
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
                                ind.extend(self.reader.slice[start..end].to_vec());
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
