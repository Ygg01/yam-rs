use alloc::borrow::ToOwned;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use yam_common::TokenType::{BlockEnd, StreamEnd};
use yam_common::{Marker, TokenType, YamlError, YamlResult};

pub trait Source {}

pub struct Token<'input> {
    token_type: TokenType<'input>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SimpleKey {
    possible: bool,
    token_number: usize,
    marker: Marker,
}

#[derive(Clone, Debug, Default)]
struct Indent {
    indent: u32,
    needs_block_end: bool,
}

pub struct Scanner<'input, S> {
    src: S,
    mark: Marker,
    tokens: VecDeque<Token<'input>>,
    error: Option<YamlError>,

    simple_keys: Vec<SimpleKey>,
    stream_end_reached: bool,
    tokens_available: bool,
    stream_start_produced: bool,
    flow_level: u32,
    indent: u32,
    indents: Vec<Indent>,
}

impl<'input, S: Source> Scanner<'input, S> {
    pub fn new(src: S) -> Scanner<'input, S> {
        Scanner {
            src,
            mark: Marker::default(),
            tokens: VecDeque::new(),
            error: None,

            simple_keys: Vec::new(),
            indents: Vec::new(),
            stream_end_reached: false,
            tokens_available: false,
            stream_start_produced: false,
            flow_level: 0,
            indent: 0,
        }
    }
    pub(crate) fn next_token(&mut self) -> YamlResult<Token<'input>> {
        if self.stream_end_reached {
            return Ok(Token {
                token_type: StreamEnd,
            });
        }

        if !self.tokens_available {
            self.fetch_more_tokens()?;
        }

        let tok = match self.tokens.pop_front() {
            Some(tok) => {
                if tok.token_type == StreamEnd {
                    self.stream_end_reached = true;
                }
                Ok(tok)
            }
            None => return Err(YamlError::UnexpectedEof),
        };

        self.tokens_available = false;

        tok
    }

    fn fetch_more_tokens(&mut self) -> YamlResult<()> {
        let mut need_more;
        loop {
            if self.tokens.is_empty() {
                need_more = true;
            } else {
                need_more = false;
                self.stale_simple_keys()?;
            }

            if !need_more {
                break;
            }

            self.fetch_next_token()?;
        }
        self.tokens_available = true;

        Ok(())
    }

    fn stale_simple_keys(&mut self) -> YamlResult<()> {
        for sk in &mut self.simple_keys {
            if sk.possible {
                return Err(YamlError::ScannerErr {
                    mark: self.mark,
                    info: "simple key expect `:`".to_owned(),
                });
            }
        }
        Ok(())
    }

    pub fn fetch_next_token(&mut self) -> YamlResult<()> {
        if !self.stream_start_produced {
            self.fetch_stream_start();
            return Ok(());
        }

        self.stale_simple_keys()?;

        let mark = self.mark;
        self.unroll_indent(mark.col);
        todo!("Implement");
        Ok(())
    }

    fn fetch_stream_start(&self) {
        todo!()
    }

    fn unroll_indent(&mut self, col: u32) {
        if self.flow_level > 0 {
            return;
        }

        while self.indent >= col {
            // TODO: avoid unwrap
            let indent = self.indents.pop().unwrap();
            self.indent = indent.indent;
            if indent.needs_block_end {
                self.tokens.push_back(Token {
                    token_type: BlockEnd,
                })
            }
        }
    }
}

impl<'input, S: Source> Iterator for Scanner<'input, S> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_some() || self.stream_end_reached {
            return None;
        }
        match self.next_token() {
            Ok(tok) => Some(tok),
            Err(e) => {
                self.error = Some(e);
                None
            }
        }
    }
}
