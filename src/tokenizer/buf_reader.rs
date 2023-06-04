use std::{io::BufRead, marker::PhantomData};

use super::Reader;

pub struct BufReader<'a, I> {
    pub input: PhantomData<I>,
    pub buf: &'a mut Vec<u8>,
    pub(crate) pos: usize,
    pub(crate) col: u32,
    pub(crate) line: u32,
}

impl<'a, S: BufRead> Reader<S> for BufReader<'a, S> {
    fn eof(&self) -> bool {
        todo!()
    }

    fn col(&self) -> u32 {
        self.col
    }

    fn line(&self) -> u32 {
        self.line
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn peek_chars(&self, _buf: &mut S) -> &[u8] {
        todo!()
    }

    fn peek_byte_at(&self, _offset: usize) -> Option<u8> {
        todo!()
    }

    fn skip_space_tab(&mut self) -> usize {
        todo!()
    }

    fn consume_bytes(&mut self, _amount: usize) -> usize {
        todo!()
    }

    fn try_read_slice_exact(&mut self, _needle: &str) -> bool {
        todo!()
    }

    fn read_line(&mut self) -> (usize, usize) {
        todo!()
    }

    fn count_spaces(&self) -> u32 {
        todo!()
    }

    fn count_spaces_till(&self, _indent: u32) -> usize {
        todo!()
    }

    fn is_empty_newline(&self) -> bool {
        todo!()
    }

    fn read_plain_one_line(
        &mut self,
        _offset_start: Option<usize>,
        _had_comment: &mut bool,
        _in_flow_collection: bool,
    ) -> (usize, usize, Option<super::ErrorType>) {
        todo!()
    }

    fn skip_detect_space_tab(&mut self, _has_tab: &mut bool) {
        todo!()
    }

    fn consume_anchor_alias(&mut self) -> (usize, usize) {
        todo!()
    }

    fn read_tag(&mut self) -> (Option<super::ErrorType>, usize, usize, usize) {
        todo!()
    }

    fn read_tag_handle(&mut self) -> Result<Vec<u8>, super::ErrorType> {
        todo!()
    }

    fn read_tag_uri(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn read_break(&mut self) -> Option<(usize, usize)> {
        todo!()
    }

    fn get_double_quote(&self, _buf: &mut S) -> Option<usize> {
        todo!()
    }

    fn get_double_quote_trim(&self, _buf: &mut S, _start_str: usize) -> Option<(usize, usize)> {
        todo!()
    }

    fn get_single_quote(&self, _buf: &mut S) -> Option<usize> {
        todo!()
    }

    fn get_single_quote_trim(&self, _buf: &mut S, _start_str: usize) -> Option<(usize, usize)> {
        todo!()
    }
}
