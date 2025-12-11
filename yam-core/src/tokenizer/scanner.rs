use alloc::borrow::ToOwned;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use yam_common::TokenType::{BlockEnd, StreamEnd};
use yam_common::{Marker, ScanResult, TokenType, YamlError, YamlResult};

pub trait Source {}

pub struct Token<'input> {
    token_type: TokenType<'input>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SimpleKey {
    possible: bool,
    required: bool,
    token_number: usize,
    marker: Marker,
}

impl SimpleKey {
    fn new(marker: Marker) -> SimpleKey {
        SimpleKey {
            possible: false,
            required: false,
            token_number: 0,
            marker,
        }
    }
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
    indents: Vec<Indent>,
    stream_end_reached: bool,
    tokens_available: bool,
    simple_key_allowed: bool,
    stream_start_produced: bool,
    leading_whitespace: bool,

    tokens_parsed: usize,
    flow_level: u32,
    indent: u32,
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
            simple_key_allowed: true,
            leading_whitespace: true,

            flow_level: 0,
            indent: 0,
            tokens_parsed: 0,
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

    fn fetch_more_tokens(&mut self) -> ScanResult {
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

    fn stale_simple_keys(&mut self) -> ScanResult {
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

    fn remove_simple_key(&mut self) -> ScanResult {
        let last = self.simple_keys.last_mut().unwrap();
        if last.possible && last.required {
            return Err(YamlError::scanner_err(self.mark, "simple key expected"));
        }

        last.possible = false;
        Ok(())
    }

    pub fn fetch_next_token(&mut self) -> ScanResult {
        if !self.stream_start_produced {
            self.fetch_stream_start();
            return Ok(());
        }

        self.stale_simple_keys()?;

        let mark = self.mark;
        self.unroll_indent(mark.col);

        if self.mark.col == 0
            && let Some(res) = self.process_start()
        {
            return res;
        }
        todo!("Implement");
        Ok(())
    }

    fn process_start(&mut self) -> Option<ScanResult> {
        if self.next_char_is(b'%') {
            Some(self.fetch_directive())
        } else if self.next_is_document_start() {
            Some(self.fetch_document_indicator(TokenType::DocumentStart))
        } else if self.next_is_document_end() {
            Some(self.finish_document())
        } else {
            None
        }
    }

    fn next_char_is(&mut self, chr: u8) -> bool {
        todo!()
    }

    fn next_is_document_start(&mut self) -> bool {
        todo!()
    }

    fn next_is_document_end(&mut self) -> bool {
        todo!()
    }

    fn fetch_document_indicator(&mut self, token_type: TokenType<'input>) -> ScanResult {
        self.unroll_indent(0);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;

        let mark = self.mark;

        self.skip_n_non_blank(3);

        self.tokens.push_back(Token { token_type });
        Ok(())
    }

    #[inline]
    fn skip_n_non_blank(&mut self, count: usize) {
        // self.input.skip_n(count);

        self.mark.pos += count;
        self.mark.col += count as u32;
        self.leading_whitespace = false;
    }

    fn fetch_directive(&mut self) -> ScanResult {
        self.unroll_indent(0);
        self.remove_simple_key()?;

        self.simple_key_allowed = false;

        let tok = self.scan_directive()?;
        self.tokens.push_back(tok);

        Ok(())
    }

    fn scan_directive(&mut self) -> YamlResult<Token<'input>> {
        todo!()
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

    fn roll_indent(
        &mut self,
        col: u32,
        number: Option<u32>,
        token_type: TokenType<'input>,
        _mark: Marker,
    ) {
        if self.flow_level > 0 {
            return;
        }

        if self.indent <= col
            && let Some(indent) = self.indents.last()
            && !indent.needs_block_end
        {
            self.indent = indent.indent;
            self.indents.pop();
        }

        if self.indent < col {
            self.indents.push(Indent {
                indent: self.indent,
                needs_block_end: true,
            });
            self.indent = col;
            match number {
                Some(n) => self.insert_token(n as usize - self.tokens_parsed, Token { token_type }),
                None => self.tokens.push_back(Token { token_type }),
            }
        }
    }

    fn insert_token(&self, _pos: usize, _token: Token) {
        todo!()
    }

    fn increase_flow_level(&mut self) -> ScanResult {
        self.simple_keys.push(SimpleKey::new(Marker {
            pos: 0,
            col: 0,
            line: 0,
        }));
        self.flow_level = self
            .flow_level
            .checked_add(1)
            .ok_or_else(|| YamlError::scanner_err(self.mark, "recursion limit exceeded"))?;
        Ok(())
    }

    fn decrease_flow_level(&mut self) {
        if self.flow_level > 0 {
            self.flow_level -= 1;
            self.simple_keys.pop().unwrap();
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
