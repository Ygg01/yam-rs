use std::{collections::VecDeque, fmt::Write, str::from_utf8_unchecked};

use crate::{tokenizer::SpanToken, Spanner};

use super::StrReader;

pub struct EventIterator<'a> {
    pub(crate) reader: StrReader<'a>,
    pub(crate) all_events: VecDeque<usize>,
    pub indent: usize,
}

impl<'a> EventIterator<'a> {
    pub fn new_from_string(input: &str) -> EventIterator {
        let mut reader = StrReader::new(input);
        let mut spanner = Spanner::default();
        while !spanner.stream_end {
            spanner.fetch_next_token(&mut reader)
        }
        EventIterator {
            reader,
            all_events: spanner.tokens,
            indent: 1,
        }
    }
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = String;

    #[allow(unused_must_use)]
    fn next(&mut self) -> Option<Self::Item> {
        pub use crate::tokenizer::SpanToken::*;

        let mut i = 0;

        let ret_val = loop {
            if i >= self.all_events.len() {
                break None;
            }
            let mut line = String::from('\n');
            let token = self.all_events[i].into();

            match (
                token,
                self.all_events.get(i + 1).copied(),
                self.all_events.get(i + 2).copied(),
            ) {
                (DirectiveYaml, Some(start), Some(end)) => {
                    line.push_str(&" ".repeat(self.indent));
                    line.push_str("%YAML ");
                    line.push_str(unsafe {
                        from_utf8_unchecked(
                            &self.reader.slice[start..end],
                        )
                    });
                    i += 3;
                    break Some(line);
                }
                (MappingStart, _, _) | (SequenceStart, _, _) | (DocumentStart, _, _) => {
                    line.push_str(&" ".repeat(self.indent));
                    self.indent += 1;
                    write!(line, "{}", token);
                    i += 1;
                    break Some(line);
                }
                (MappingEnd, _, _) | (SequenceEnd, _, _) | (DocumentEnd, _, _) => {
                    self.indent -= 1;
                    line.push_str(&" ".repeat(self.indent));
                    write!(line, "{}", token);
                    i += 1;
                    break Some(line);
                }
                (Error, _, _) => {
                    line.push_str(&" ".repeat(self.indent));
                    i += 1;
                    write!(line, "ERR");
                    break Some(line);
                }
                (Mark, _, _) => {
                    line.push_str(&" ".repeat(self.indent));
                    line.push_str("=VAL ");
                    let mut x = String::new();

                    loop {
                        match (
                            self.all_events.get(i).map(Into::<SpanToken>::into),
                            self.all_events.get(i + 1).copied(),
                        ) {
                            (Some(Space), _) => {
                                x.push(' ');
                                i += 1;
                            }
                            (Some(NewLine), Some(len)) => {
                                x.push_str(&"\\n".repeat(len));
                                i += 2;
                            }
                            (Some(Mark), Some(end)) if end < NewLine as usize => {
                                x.push_str(unsafe {
                                    from_utf8_unchecked(
                                        &self.reader.slice
                                            [self.all_events[i]..self.all_events[i + 1]],
                                    )
                                });
                                i += 2;
                            }
                            (Some(KeyEnd), _) | (Some(Separator), _) => {
                                i += 1;
                                break;
                            }
                            _ => break,
                        }
                    }

                    line.push_str(&x);
                    break Some(line);
                }
                _ => {
                    i += 1;
                }
            };
        };
        self.all_events.drain(0..i);

        ret_val
    }
}
