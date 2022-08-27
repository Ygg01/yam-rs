use std::io;
use std::io::BufRead;
use crate::error::{YamlError, YamlResult};
use crate::tokenizer::tokenizer::{SpanToken, YamlTokenizer};
use crate::tokenizer::YamlToken;

pub struct StrReader<'a> {
    pub slice: &'a str,
    pos: usize,
}


pub(crate) trait Reader {
    fn peek_byte(&mut self) -> YamlResult<Option<u8>>;
    fn consume_bytes(&mut self, amount: usize);
    fn slice_bytes(&self, start: usize, end: usize) -> &[u8];
    fn append_curr_char(&mut self) -> usize;

    fn try_read_slice(&mut self, needle: &str, case_sensitive: bool) -> bool;
    #[inline(always)]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        self.try_read_slice(needle, true)
    }
    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead;
}

impl<'b> Reader for StrReader<'b>
{
    fn peek_byte(&mut self) -> YamlResult<Option<u8>> {
        todo!()
    }

    fn consume_bytes(&mut self, amount: usize) {
        todo!()
    }

    fn slice_bytes(&self, start: usize, end: usize) -> &[u8] {
        todo!()
    }

    fn append_curr_char(&mut self) -> usize {
        todo!()
    }

    fn try_read_slice(&mut self, needle: &str, case_sensitive: bool) -> bool {
        todo!()
    }

    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead {
        todo!()
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FastRead {
    Char(u8),
    InterNeedle(usize, usize),
    EOF,
}
