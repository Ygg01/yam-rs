use std::cmp::min;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::ops::ControlFlow::{Break, Continue};
use std::slice::Iter;
use ErrorKind::Interrupted;

use yam_core::tokenizer::{DirectiveState, ErrorType, LexMutState, Reader};

pub struct BufferedReader<B, S> {
    buffer: B,
    source: S,
    temp_buf: Vec<u8>,
    col: u32,
    line: u32,
    abs_pos: usize,
}

impl<B: Default, S: Default> Default for BufferedReader<B, S> {
    fn default() -> Self {
        Self {
            buffer: B::default(),
            source: S::default(),
            temp_buf: Vec::with_capacity(3),
            col: u32::default(),
            line: u32::default(),
            abs_pos: usize::default(),
        }
    }
}

impl<'a, S: BufRead> Reader for BufferedReader<&'a mut Vec<u8>, S> {
    fn eof(&mut self) -> bool {
        !matches!(self.source.fill_buf(), Ok(b) if !b.is_empty())
    }

    fn col(&self) -> u32 {
        self.col
    }

    fn line(&self) -> u32 {
        self.line
    }

    fn offset(&self) -> usize {
        todo!()
    }

    fn peek_chars(&mut self) -> &[u8] {
        self.temp_buf.clear();
        loop {
            break match self.source.fill_buf() {
                Ok(n) => {
                    let min = min(n.len(), 2);
                    self.temp_buf.extend(&n[0..min]);
                }
                Err(e) if e.kind() == Interrupted => continue,
                Err(_) => break,
            };
        }
        &self.temp_buf
    }

    fn peek_byte_at(&mut self, _offset: usize) -> Option<u8> {
        loop {
            break match self.source.fill_buf() {
                Ok(n) if n.len() <= _offset => None,
                Ok(n) => Some(n[0]),
                Err(e) if e.kind() == Interrupted => continue,
                Err(_) => None,
            };
        }
    }

    fn peek_stream_ending(&mut self) -> bool {
        loop {
            break match self.source.fill_buf() {
                Ok(n) => {
                    (n.starts_with(b"...") || n.starts_with(b"---"))
                        && self.col == 0
                        && n.get(3).map_or(true, |c| {
                            *c == b'\t'
                                || *c == b' '
                                || *c == b'\r'
                                || *c == b'\n'
                                || *c == b'['
                                || *c == b'{'
                        })
                }
                Err(e) if e.kind() == Interrupted => continue,
                Err(_) => false,
            };
        }
    }

    fn skip_space_tab(&mut self) -> usize {
        let mut amount = 0;
        loop {
            let (done, used) = {
                let available = match self.source.fill_buf() {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == Interrupted => continue,
                    Err(_) => break,
                };
                match available.iter().try_fold(0usize, |pos, chr| {
                    if *chr == b' ' || *chr == b'\t' {
                        Continue(pos + 1)
                    } else {
                        Break(pos)
                    }
                }) {
                    Continue(x) => (false, x),
                    Break(x) => (true, x),
                }
            };
            amount += used;
            self.skip_bytes(used);
            if done || used == 0 {
                break;
            }
        }
        amount
    }

    fn skip_bytes(&mut self, amount: usize) -> usize {
        self.source.consume(amount);
        self.col += TryInto::<u32>::try_into(amount).expect("Amount of indents can't exceed u32");
        self.abs_pos += amount;
        self.abs_pos
    }

    fn save_bytes(
        &mut self,
        _tokens: &mut Vec<usize>,
        _start: usize,
        _end: usize,
        _newline: Option<u32>,
    ) {
        todo!()
    }

    fn emit_tokens(
        &mut self,
        _tokens: &mut Vec<usize>,
        _start: usize,
        _end: usize,
        _new_lines: u32,
    ) {
        todo!()
    }

    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        let needle_bytes = needle.as_bytes();
        let mut buf = Vec::with_capacity(needle_bytes.len());
        loop {
            match self.source.fill_buf() {
                Ok([]) => break,
                Ok(n) => buf.extend(n),
                Err(ref e) if e.kind() == Interrupted => continue,
                Err(_) => break,
            };
            if buf.len() >= needle_bytes.len() {
                break;
            }
        }
        let result = buf.starts_with(needle_bytes);
        self.skip_bytes(needle_bytes.len());
        result
    }

    fn get_read_line(&mut self) -> (usize, usize, usize) {
        todo!()
    }

    #[inline]
    fn read_line(&mut self, _space_indent: &mut Option<u32>) -> (usize, usize) {
        todo!()
    }

    fn count_spaces(&mut self) -> u32 {
        todo!()
    }

    fn count_whitespace_from(&mut self, _offset: usize) -> usize {
        todo!()
    }

    fn count_spaces_till(&mut self, _indent: u32) -> usize {
        todo!()
    }

    fn is_empty_newline(&mut self) -> bool {
        todo!()
    }

    fn count_space_then_tab(&mut self) -> (u32, u32) {
        todo!()
    }

    fn consume_anchor_alias(&mut self) -> (usize, usize) {
        todo!()
    }

    fn read_tag(&mut self, _lexer_state: &mut LexMutState) -> (usize, usize, usize) {
        todo!()
    }

    fn read_tag_handle(&mut self, _space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType> {
        todo!()
    }

    fn read_tag_uri(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn read_directive(
        &mut self,
        _directive_state: &mut DirectiveState,
        _lexer_state: &mut LexMutState,
    ) -> bool {
        todo!()
    }

    fn read_break(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn emit_new_space(&mut self, _tokens: &mut Vec<usize>, _new_lines: &mut Option<usize>) {
        todo!()
    }

    fn read_plain_one_line(
        &mut self,
        _in_flow_collection: bool,
        _had_comment: &mut bool,
        _lexer_state: &mut LexMutState,
    ) -> (usize, usize) {
        todo!()
    }

    fn get_quote_line_offset(&mut self, quote: u8) -> &[u8] {
        todo!()
    }

    fn save_to_buf(&mut self, start: usize, input: &[u8]) -> (usize, usize) {
        todo!()
    }

    fn get_zero_slice(&mut self) -> Vec<u8> {
        todo!()
    }
}

// Test only helper
struct StringReader<'a> {
    iter: Iter<'a, u8>,
}

impl<'a> StringReader<'a> {
    /// Wrap a string in a `StringReader`, which implements `std::io::Read`.
    pub fn new(data: &'a str) -> Self {
        Self {
            iter: data.as_bytes().iter(),
        }
    }
}

impl<'a> Read for StringReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..<[u8]>::len(buf) {
            if let Some(x) = self.iter.next() {
                buf[i] = *x;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}
#[test]
fn test_skip_space_tabs() {
    let mut buffer = Vec::<u8>::with_capacity(10);
    let mut buff_reader = BufferedReader {
        buffer: &mut buffer,
        source: BufReader::new(StringReader::new("   \t test")),
        temp_buf: vec![],
        col: 0,
        line: 0,
        abs_pos: 0,
    };
    assert_eq!(buff_reader.skip_space_tab(), 5);
    assert!(buff_reader.peek_byte_is(b't'));
    assert_eq!(buff_reader.col(), 5);
    assert_eq!(buff_reader.abs_pos, 5)
}

#[test]
fn test_read_exact() {
    let mut buffer = Vec::<u8>::with_capacity(10);
    let mut buff_reader = BufferedReader {
        buffer: &mut buffer,
        source: BufReader::new(StringReader::new("%YAML 1.2")),
        temp_buf: vec![],
        col: 0,
        line: 0,
        abs_pos: 0,
    };
    assert!(buff_reader.try_read_slice_exact("%YAML"));
    assert!(buff_reader.peek_byte_is(b' '));
    assert_eq!(buff_reader.col(), 5);
    assert_eq!(buff_reader.abs_pos, 5)
}
