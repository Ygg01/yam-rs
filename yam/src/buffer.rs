use std::io::BufRead;
use yam_core::tokenizer::{DirectiveState, ErrorType, LexMutState, Reader};

pub struct BufReader<B, S> {
    _buffer: B,
    source: S,
    col: u32,
    line: u32,
    _buffer_pos: usize,
}

impl<R, S: BufRead> Reader for BufReader<R, S> {
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
        todo!()
    }

    fn peek_byte_at(&mut self, _offset: usize) -> Option<u8> {
        todo!()
    }

    fn peek_stream_ending(&mut self) -> bool {
        todo!()
    }

    fn skip_space_tab(&mut self) -> usize {
        todo!()
    }

    fn skip_space_and_tab_detect(&mut self, _has_tab: &mut bool) -> usize {
        todo!()
    }

    fn skip_bytes(&mut self, amount: usize) -> usize {
        self.source.consume(amount);
        self._buffer_pos += amount;
        self._buffer_pos
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

    fn read_tag(&mut self, lexer_state: &mut LexMutState) -> (usize, usize, usize) {
        todo!()
    }

    fn read_tag_handle(&mut self, _space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType> {
        todo!()
    }

    fn read_tag_uri(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn read_directive(&mut self, directive_state: &mut DirectiveState, lexer_state: &mut LexMutState) -> bool {
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
