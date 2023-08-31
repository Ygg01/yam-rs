use std::cmp::min;
use std::io;
use std::io::BufRead;
use yam_core::tokenizer::{DirectiveState, ErrorType, LexMutState, Reader};

pub struct BufReader<B, S> {
    buffer: B,
    source: S,
    temp_buf: Vec<u8>,
    col: u32,
    line: u32,
    abs_pos: usize,
}

impl<B: Default, S: Default> Default for BufReader<B, S> {
    fn default() -> Self {
        Self {
            buffer: B::default(),
            source: S::default(),
            // This will never allocate more than 3
            temp_buf: Vec::with_capacity(3),
            col: u32::default(),
            line: u32::default(),
            abs_pos: usize::default(),
        }
    }
}

impl<'a, S: BufRead> Reader for BufReader<&'a mut Vec<u8>, S> {
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
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
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
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
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
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => false,
            };
        }
    }

    fn skip_space_tab(&mut self) -> usize {
        todo!()
    }

    fn skip_space_and_tab_detect(&mut self, _has_tab: &mut bool) -> usize {
        todo!()
    }

    fn skip_bytes(&mut self, amount: usize) -> usize {
        self.source.consume(amount);
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

    fn try_read_slice_exact(&mut self, _needle: &str) -> bool {
        todo!()
    }

    fn get_read_line(&mut self) -> (usize, usize, usize) {
        todo!()
    }

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
        _offset_start: Option<usize>,
        _had_comment: &mut bool,
        _in_flow_collection: bool,
    ) -> (usize, usize, usize) {
        todo!()
    }

    fn get_quote_line_offset(&mut self, _quote: u8) -> &[u8] {
        todo!()
    }
}
