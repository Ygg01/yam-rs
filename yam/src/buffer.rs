use std::io::BufRead;
use yam_core::tokenizer::{ErrorType, Reader};

pub struct BufReader<B, S> {
    _buffer: B,
    source: S,
    col: u32,
    pos: u32,
    _buffer_pos: usize,

}

impl<R, S: BufRead> Reader<R> for BufReader<R, S> {
    fn eof(&mut self) -> bool {
        !matches!(self.source.fill_buf(), Ok(b) if !b.is_empty())
    }

    fn col(&self) -> u32 {
        self.col
    }

    fn line(&self) -> u32 {
        self.pos
    }

    fn offset(&self) -> usize {
        todo!()
    }

    fn peek_chars(&mut self) -> &[u8] {
        todo!()
    }

    fn peek_two_chars(&mut self) -> &[u8] {
        todo!()
    }

    fn peek_byte_at(&mut self, offset: usize) -> Option<u8> {
        todo!()
    }

    fn skip_space_tab(&mut self) -> usize {
        todo!()
    }

    fn skip_space_and_tab_detect(&mut self, has_tab: &mut bool) -> usize {
        todo!()
    }

    fn skip_bytes(&mut self, amount: usize) -> usize {
        todo!()
    }

    fn save_bytes(&mut self, amount: usize) -> usize {
        todo!()
    }


    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        todo!()
    }

    fn get_read_line(&mut self) -> (usize, usize, usize) {
        todo!()
    }

    fn read_line(&mut self) -> (usize, usize) {
        todo!()
    }

    fn count_spaces(&mut self) -> u32 {
        todo!()
    }

    fn count_whitespace_from(&mut self, offset: usize) -> usize {
        todo!()
    }

    fn count_spaces_till(&mut self, indent: u32) -> usize {
        todo!()
    }

    fn is_empty_newline(&mut self) -> bool {
        todo!()
    }

    fn get_double_quote(&mut self) -> Option<usize> {
        todo!()
    }

    fn get_double_quote_trim(&mut self, start_str: usize) -> Option<(usize, usize)> {
        todo!()
    }

    fn get_single_quote(&mut self) -> Option<usize> {
        todo!()
    }

    fn get_single_quote_trim(&mut self, start_str: usize) -> Option<(usize, usize)> {
        todo!()
    }

    fn count_space_then_tab(&mut self) -> (u32, u32) {
        todo!()
    }

    fn consume_anchor_alias(&mut self) -> (usize, usize) {
        todo!()
    }

    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize) {
        todo!()
    }

    fn read_tag_handle(&mut self) -> Result<Vec<u8>, ErrorType> {
        todo!()
    }

    fn read_tag_uri(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn read_break(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn read_plain_one_line(&mut self, offset_start: Option<usize>, had_comment: &mut bool, in_flow_collection: bool) -> (usize, usize, usize) {
        todo!()
    }
}

