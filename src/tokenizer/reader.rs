use crate::error::{YamlError, YamlResult};
use crate::tokenizer::tokenizer::{SpanToken, YamlTokenizer};
use crate::tokenizer::YamlToken;
use std::io;
use std::io::BufRead;

pub struct StrReader<'a> {
    pub slice: &'a str,
    pos: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self { slice, pos: 0 }
    }
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
    fn skip_space_tab(&mut self) -> usize;
    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead;
}

impl<'r> Reader for StrReader<'r> {
    fn peek_byte(&mut self) -> YamlResult<Option<u8>> {
        match self.slice.as_bytes().get(self.pos) {
            Some(x) => Ok(Some(*x)),
            _ => Err(YamlError::UnexpectedEof),
        }
    }

    fn consume_bytes(&mut self, amount: usize) {
        self.pos += amount;
    }

    fn slice_bytes(&self, start: usize, end: usize) -> &'r [u8] {
        &self.slice.as_bytes()[start..end]
    }

    fn append_curr_char(&mut self) -> usize {
        self.pos
    }

    fn try_read_slice(&mut self, needle: &str, case_sensitive: bool) -> bool {
        if self.slice.len() < needle.len() {
            return false;
        }

        let read = if case_sensitive {
            self.slice.as_bytes()[self.pos..self.pos + needle.len()].starts_with(needle.as_bytes())
        } else {
            needle.as_bytes().iter().enumerate().all(|(offset, char)| {
                self.slice.as_bytes()[self.pos + offset].to_ascii_lowercase() == char.to_ascii_lowercase()
            })
        };

        if read {
            self.pos += needle.len();
        }
        read
    }

    fn skip_space_tab(&mut self) -> usize {
        let start = self.slice.as_bytes()
            .iter()
            .position(|b| !is_tab_space(*b))
            .unwrap_or(0);
        start
    }

    fn read_fast_until(&mut self, needle: &[u8]) -> FastRead {
        let (read, n) = match fast_find(needle, &self.slice.as_bytes()[self.pos..]) {
            Some(0) => (FastRead::Char(self.slice.as_bytes()[self.pos]), 1),
            Some(size) => (FastRead::InterNeedle(self.pos, self.pos + size), size),
            None => (FastRead::EOF, 0),
        };
        self.pos += n;
        read
    }
}

#[inline]
pub(crate) fn is_tab_space(b: u8) -> bool {
    match b {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn fast_find(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    #[cfg(feature = "jetscii")]
    {
        debug_assert!(needle.len() <= 16);
        let mut needle_arr = [0; 16];
        needle_arr[..needle.len()].copy_from_slice(needle);
        jetscii::Bytes::new(needle_arr, needle.len() as i32, |b| needle.contains(&b)).find(haystack)
    }

    #[cfg(not(feature = "jetscii"))]
    {
        haystack.iter().position(|b| needle.contains(b))
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum FastRead {
    Char(u8),
    InterNeedle(usize, usize),
    EOF,
}
