use crate::tokenizer::char_utils::{is_blank_or_break, is_flow};
use alloc::borrow::ToOwned;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::vec::Vec;
use yam_common::TokenType::{BlockEnd, FlowMappingEnd, FlowSequenceStart, StreamEnd};
use yam_common::{Marker, ScanResult, TokenType, YamlError, YamlResult};

pub trait Source {
    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> (u32, Result<SkipTabs, &'static str>);
    fn next_is_breakz(&self) -> bool;

    fn peek_two(&self) -> [u8; 2];

    fn peek_char(&self) -> char;

    fn peek(&self) -> u8;

    fn next_is_three_and_break(&self, chr: u8) -> bool;
}

enum SkipTabs {
    Yes,
    No,
}

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

#[derive(Debug, PartialEq)]
enum ImplicitMappingState {
    /// It is possible there is an implicit mapping.
    ///
    /// This state is the one when we have just encountered the opening `[`. We need more context
    /// to know whether an implicit mapping follows.
    Possible,
    /// We are inside the implcit mapping.
    ///
    /// Note that this state is not set immediately (we need to have encountered the `:` to know).
    Inside,
}

pub struct Scanner<'input, S> {
    src: S,
    mark: Marker,
    tokens: VecDeque<Token<'input>>,
    error: Option<YamlError>,

    simple_keys: Vec<SimpleKey>,
    indents: Vec<Indent>,
    implicit_flow_mapping_states: Vec<ImplicitMappingState>,
    stream_end_reached: bool,
    tokens_available: bool,
    simple_key_allowed: bool,
    stream_start_produced: bool,
    leading_whitespace: bool,
    flow_mapping_started: bool,

    adjacent_value_allowed_at: usize,
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
            implicit_flow_mapping_states: Vec::new(),
            error: None,

            simple_keys: Vec::new(),
            indents: Vec::new(),
            stream_end_reached: false,
            tokens_available: false,
            stream_start_produced: false,
            simple_key_allowed: true,
            leading_whitespace: true,
            flow_mapping_started: false,

            adjacent_value_allowed_at: 0,
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
        self.src.peek() == chr
    }

    fn next_is_document_start(&mut self) -> bool {
        self.src.next_is_three_and_break(b'-')
    }

    fn next_is_document_end(&mut self) -> bool {
        self.src.next_is_three_and_break(b'.')
    }

    fn fetch_next_token(&mut self) -> ScanResult {
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

        if self.mark.col < self.indent {
            return Err(YamlError::scanner_err(self.mark, "invalid indentation"));
        }

        self.fetch_main_loop()
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

    fn fetch_stream_start(&mut self) {
        let mark = self.mark;
        self.indent = 0;
        self.stream_start_produced = true;
        self.simple_key_allowed = true;
        self.tokens.push_back(Token {
            token_type: TokenType::StreamStart,
        });
        self.simple_keys.push(SimpleKey::new(Marker::default()));
    }

    fn fetch_main_loop(&mut self) -> ScanResult {
        let c = self.src.peek_two();
        match c {
            [b'[', _] => self.fetch_flow_collection_start(TokenType::FlowSequenceStart),
            [b'{', _] => self.fetch_flow_collection_start(TokenType::FlowMappingStart),
            [b']', _] => self.fetch_flow_collection_end(TokenType::FlowSequenceEnd),
            [b'}', _] => self.fetch_flow_collection_end(TokenType::FlowMappingEnd),
            [b',', _] => self.fetch_flow_entry(),
            [b'-', x] if is_blank_or_break(x) => self.fetch_block_entry(),
            [b'?', x] if is_blank_or_break(x) => self.fetch_key(),
            [b':', x] if is_blank_or_break(x) => self.fetch_value(),
            [b':', x]
                if self.flow_level > 0
                    && (is_flow(x) || self.mark.pos == self.adjacent_value_allowed_at) =>
            {
                self.fetch_flow_value()
            }

            [b'*', _] => self.fetch_anchor(true),
            [b'&', _] => self.fetch_anchor(false),
            [b'!', _] => self.fetch_tag(),
            [b'|', _] if self.flow_level == 0 => self.fetch_block_scalar(true),
            [b'>', _] if self.flow_level == 0 => self.fetch_block_scalar(false),
            [b'\'', _] => self.fetch_flow_scalar(true),
            [b'"', _] => self.fetch_flow_scalar(false),
            [b'-', x] if !is_blank_or_break(x) => self.fetch_plain_scalar(),
            [b':' | b'?', x] if !is_blank_or_break(x) && self.flow_level == 0 => {
                self.fetch_plain_scalar()
            }
            [b'%' | b'@' | b'`', _] => {
                let chr = self.src.peek_char();
                Err(YamlError::scanner_err(
                    self.mark,
                    &format!("Unexpected character `{chr}`"),
                ))
            }
            _ => self.fetch_plain_scalar(),
        }
    }

    fn fetch_flow_collection_start(&mut self, token_type: TokenType<'input>) -> ScanResult {
        self.save_simple_key();

        self.roll_one_col_indent();
        self.increase_flow_level()?;

        self.simple_key_allowed = true;

        let start_mark = self.mark;
        self.skip_non_blank();

        if token_type == FlowSequenceStart {
            self.flow_mapping_started = true;
        } else {
            self.implicit_flow_mapping_states
                .push(ImplicitMappingState::Possible);
        }

        self.skip_ws_to_eol(SkipTabs::Yes)?;

        self.tokens.push_back(Token { token_type });

        Ok(())
    }

    fn fetch_flow_collection_end(&mut self, token_type: TokenType<'input>) -> ScanResult {
        self.remove_simple_key()?;
        self.decrease_flow_level();

        self.simple_key_allowed = false;

        if matches!(token_type, TokenType::FlowMappingEnd) {
            self.end_implicit_mapping(self.mark);
            self.implicit_flow_mapping_states.pop();
        }

        let start_mark = self.mark;
        self.skip_non_blank();
        self.skip_ws_to_eol(SkipTabs::Yes)?;

        if self.flow_level > 0 {
            self.adjacent_value_allowed_at = self.mark.pos;
        }

        self.tokens.push_back(Token { token_type });
        Ok(())
    }

    fn end_implicit_mapping(&mut self, _mark: Marker) {
        if let Some(implicit_mapping) = self.implicit_flow_mapping_states.last_mut()
            && *implicit_mapping == ImplicitMappingState::Inside
        {
            self.flow_mapping_started = false;
            *implicit_mapping = ImplicitMappingState::Possible;
            self.tokens.push_back(Token {
                token_type: FlowMappingEnd,
            })
        }
    }

    fn fetch_plain_scalar(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_flow_entry(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_block_entry(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_key(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_value(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_flow_value(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_anchor(&mut self, _is_alias: bool) -> ScanResult {
        todo!()
    }

    fn fetch_tag(&mut self) -> ScanResult {
        todo!()
    }

    fn fetch_block_scalar(&mut self, _is_literal: bool) -> ScanResult {
        todo!()
    }

    fn fetch_flow_scalar(&mut self, _single: bool) -> ScanResult {
        todo!()
    }

    fn fetch_directive(&mut self) -> ScanResult {
        self.unroll_indent(0);
        self.remove_simple_key()?;

        self.simple_key_allowed = false;

        let tok = self.scan_directive()?;
        self.tokens.push_back(tok);

        Ok(())
    }

    fn finish_document(&mut self) -> ScanResult {
        self.fetch_document_indicator(TokenType::DocumentEnd)?;
        self.skip_ws_to_eol(SkipTabs::Yes)?;
        if !self.src.next_is_breakz() {
            Err(YamlError::scanner_err(
                self.mark,
                "Invalid content after document end marker",
            ))
        } else {
            Ok(())
        }
    }

    fn skip_n_non_blank(&mut self, count: usize) {
        // self.input.skip_n(count);

        self.mark.pos += count;
        self.mark.col += count as u32;
        self.leading_whitespace = false;
    }

    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> Result<SkipTabs, YamlError> {
        let (n_bytes, result) = self.src.skip_ws_to_eol(skip_tabs);

        self.mark.col += n_bytes;
        self.mark.pos += n_bytes as usize;
        result.map_err(|message| YamlError::scanner_err(self.mark, message))
    }

    fn skip_non_blank(&self) {
        todo!()
    }

    fn scan_directive(&mut self) -> YamlResult<Token<'input>> {
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

    fn roll_one_col_indent(&mut self) {
        if self.flow_level == 0 && self.indents.last().is_some_and(|x| x.needs_block_end) {
            self.indents.push(Indent {
                indent: self.indent,
                needs_block_end: false,
            });
            self.indent += 1;
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

    fn save_simple_key(&mut self) {
        if self.simple_key_allowed {
            let required = self.flow_level == 0
                && self.indent == self.mark.col
                && self.indents.last().map_or(false, |x| x.needs_block_end);

            let sk = SimpleKey {
                marker: self.mark,
                required,
                possible: true,
                token_number: self.tokens_parsed + self.tokens.len(),
            };

            self.simple_keys.pop();
            self.simple_keys.push(sk);
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
