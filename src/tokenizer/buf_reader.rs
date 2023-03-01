use std::collections::VecDeque;
use std::io::BufRead;
use crate::tokenizer::{Reader, SpanToken};
use crate::tokenizer::spanner::ParserState;

pub struct BufReader<B> {
    pub slice: B,
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

impl<'a, B: BufRead> Reader<&'a mut Vec<u8>> for BufReader<B> {
    fn eof(&self) -> bool {
        todo!()
    }

    fn col(&self) -> usize {
        todo!()
    }

    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        todo!()
    }

    fn peek_byte(&self) -> Option<u8> {
        todo!()
    }

    fn count_space_tab(&self, allow_tab: bool) -> usize {
        todo!()
    }

    fn consume_bytes(&mut self, amount: usize) -> usize {
        todo!()
    }

    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        todo!()
    }

    fn read_line(&mut self) -> (usize, usize) {
        todo!()
    }

    fn read_block_seq(&mut self, indent: usize) -> Option<ParserState> {
        todo!()
    }

    fn read_single_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>) {
        todo!()
    }

    fn read_plain_scalar(&mut self, start_indent: usize, init_indent: usize, curr_state: &ParserState) -> (Vec<SpanToken>, Option<ParserState>) {
        todo!()
    }

    fn skip_separation_spaces(&mut self, allow_comments: bool) -> usize {
        todo!()
    }

    fn read_double_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>) {
        todo!()
    }

    fn read_block_scalar(&mut self, literal: bool, curr_state: &ParserState, tokens: &mut VecDeque<SpanToken>) {
        todo!()
    }

    fn try_read_yaml_directive(&mut self, tokens: &mut VecDeque<SpanToken>) {
        todo!()
    }

    fn consume_anchor_alias(&mut self, tokens: &mut VecDeque<SpanToken>, token_push: SpanToken) {
        todo!()
    }

    fn read_tag(&self) -> Option<(usize, usize)> {
        todo!()
    }
}