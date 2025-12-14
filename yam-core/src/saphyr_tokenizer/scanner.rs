use crate::saphyr_tokenizer::char_utils::*;
use crate::saphyr_tokenizer::source::Source;
use TokenType::FlowSequenceEnd;
use alloc::borrow::Cow;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use yam_common::ScalarType::Plain;
use yam_common::TokenType::{
    BlockEnd, FlowMappingEnd, FlowMappingStart, FlowSequenceStart, StreamEnd,
};
use yam_common::{
    ChompIndicator, Marker, ScalarType, ScanResult, TokenType, YamlError, YamlResult,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SkipTabs {
    Yes,
    No,
    Result { any_tabs: bool, has_yaml_ws: bool },
}

impl SkipTabs {
    pub(crate) fn found_tabs(&self) -> bool {
        matches!(self, SkipTabs::Result { any_tabs: true, .. })
    }

    #[must_use]
    pub(crate) fn has_valid_yaml_ws(&self) -> bool {
        matches!(
            self,
            SkipTabs::Result {
                has_yaml_ws: true,
                ..
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Span {
    pub start: Marker,
    pub end: Marker,
}

impl Span {
    pub fn new(start: Marker, end: Marker) -> Self {
        Span { start, end }
    }

    pub fn empty(mark: Marker) -> Self {
        Span {
            start: mark,
            end: mark,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token<'input> {
    pub span: Span,
    pub token_type: TokenType<'input>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SimpleKey {
    possible: bool,
    required: bool,
    token_number: usize,
    mark: Marker,
}

impl SimpleKey {
    fn new(mark: Marker) -> SimpleKey {
        SimpleKey {
            possible: false,
            required: false,
            token_number: 0,
            mark,
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
    pub(crate) mark: Marker,
    tokens: VecDeque<Token<'input>>,
    pub(crate) error: Option<YamlError>,

    simple_keys: Vec<SimpleKey>,
    indents: Vec<Indent>,
    implicit_flow_mapping_states: Vec<ImplicitMappingState>,
    stream_end_reached: bool,
    tokens_available: bool,
    simple_key_allowed: bool,
    pub(crate) stream_start_produced: bool,
    pub(crate) stream_end_produced: bool,
    leading_whitespace: bool,
    flow_mapping_started: bool,

    adjacent_value_allowed_at: usize,
    tokens_parsed: usize,
    flow_level: u32,
    indent: u32,

    buf_leading_break: Vec<u8>,
    buf_trailing_breaks: Vec<u8>,
    buf_whitespaces: Vec<u8>,
}

impl<'input, S: Source> Scanner<'input, S> {
    pub fn new(src: S) -> Scanner<'input, S> {
        Scanner {
            src,
            mark: Marker {
                pos: 0,
                col: 1,
                line: 1,
            },
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
            stream_end_produced: false,

            adjacent_value_allowed_at: 0,
            flow_level: 0,
            indent: 0,
            tokens_parsed: 0,

            buf_leading_break: Vec::new(),
            buf_trailing_breaks: Vec::new(),
            buf_whitespaces: Vec::new(),
        }
    }

    fn get_span(&self, start: Marker) -> Span {
        Span {
            start,
            end: self.mark,
        }
    }

    pub(crate) fn next_token(&mut self) -> YamlResult<Token<'input>> {
        if self.stream_end_produced {
            return Ok(Token {
                span: Span {
                    start: Marker::default(),
                    end: Marker::default(),
                },
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
        self.tokens_parsed += 1;

        if matches!(
            tok,
            Ok(Token {
                token_type: StreamEnd,
                ..
            })
        ) {
            self.stream_end_produced = true;
        }

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
                for sk in &self.simple_keys {
                    if sk.possible && sk.token_number == self.tokens_parsed {
                        need_more = true;
                        break;
                    }
                }
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
        let x = self.src.next_is_three(b'-') && is_blank_or_breakz(self.src.peek_nth(3));
        x
    }

    fn next_is_document_end(&mut self) -> bool {
        self.src.next_is_three(b'.') && is_blank_or_breakz(self.src.peek_nth(3))
    }

    fn fetch_next_token(&mut self) -> ScanResult {
        if !self.stream_start_produced {
            self.fetch_stream_start();
            return Ok(());
        }

        self.skip_to_next_token()?;
        self.stale_simple_keys()?;

        let mark = self.mark;
        self.unroll_indent(mark.col);

        if self.src.buf_is_empty() {
            self.fetch_stream_end()?;
            return Ok(());
        }

        if self.mark.col == 1
            && let Some(res) = self.process_start()
        {
            return res;
        }

        if self.mark.col < self.indent {
            return Err(YamlError::new_str(self.mark, "invalid indentation"));
        }

        self.fetch_main_loop()
    }

    fn fetch_stream_end(&mut self) -> ScanResult {
        // force new line
        if self.mark.col != 0 {
            self.mark.col = 1;
            self.mark.line += 1;
        }

        // If the stream ended, we won't have more context. We can stall all the simple keys we
        // had. If one was required, however, that was an error and we must propagate it.
        for sk in &mut self.simple_keys {
            if sk.required && sk.possible {
                return Err(YamlError::new_str(self.mark, "simple key expected"));
            }
            sk.possible = false;
        }

        self.unroll_indent(0);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;
        let span = Span::empty(self.mark);

        self.tokens.push_back(Token {
            span,
            token_type: TokenType::StreamEnd,
        });
        Ok(())
    }

    fn fetch_document_indicator(&mut self, token_type: TokenType<'input>) -> ScanResult {
        self.unroll_indent(0);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;

        let mark = self.mark;

        self.skip_n_non_blank(3);

        let span = Span::new(mark, self.mark);

        self.tokens.push_back(Token { span, token_type });
        Ok(())
    }

    fn fetch_stream_start(&mut self) {
        let mark = self.mark;
        self.indent = 0;
        self.stream_start_produced = true;
        self.simple_key_allowed = true;
        self.tokens.push_back(Token {
            span: Span::new(mark, self.mark),
            token_type: TokenType::StreamStart,
        });
        self.simple_keys.push(SimpleKey::new(Marker::default()));
    }

    fn fetch_main_loop(&mut self) -> ScanResult {
        let c = self.src.peek_two();
        match c {
            [b'[', _] => self.fetch_flow_collection_start(FlowSequenceStart),
            [b'{', _] => self.fetch_flow_collection_start(FlowMappingStart),
            [b']', _] => self.fetch_flow_collection_end(FlowSequenceEnd),
            [b'}', _] => self.fetch_flow_collection_end(FlowMappingEnd),
            [b',', _] => self.fetch_flow_entry(),
            [b'-', x] if is_blank_or_breakz(x) => self.fetch_block_entry(),
            [b'?', x] if is_blank_or_breakz(x) => self.fetch_key(),
            [b':', x] if is_blank_or_breakz(x) => self.fetch_value(),
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
                Err(YamlError::new_str(
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

        if token_type == FlowMappingStart {
            self.flow_mapping_started = true;
        } else {
            self.implicit_flow_mapping_states
                .push(ImplicitMappingState::Possible);
        }

        self.skip_ws_to_eol(SkipTabs::Yes)?;

        let span = self.get_span(start_mark);
        self.tokens.push_back(Token { span, token_type });

        Ok(())
    }

    fn fetch_flow_collection_end(&mut self, token_type: TokenType<'input>) -> ScanResult {
        self.remove_simple_key()?;
        self.decrease_flow_level();

        self.simple_key_allowed = false;

        if matches!(token_type, FlowSequenceEnd) {
            self.end_implicit_mapping(self.mark);
            self.implicit_flow_mapping_states.pop();
        }

        let start_mark = self.mark;
        self.skip_non_blank();
        self.skip_ws_to_eol(SkipTabs::Yes)?;

        if self.flow_level > 0 {
            self.adjacent_value_allowed_at = self.mark.pos;
        }

        let span = self.get_span(start_mark);
        self.tokens.push_back(Token { span, token_type });
        Ok(())
    }

    fn end_implicit_mapping(&mut self, mark: Marker) {
        if let Some(implicit_mapping) = self.implicit_flow_mapping_states.last_mut()
            && *implicit_mapping == ImplicitMappingState::Inside
        {
            self.flow_mapping_started = false;
            *implicit_mapping = ImplicitMappingState::Possible;
            let span = self.get_span(mark);
            self.tokens.push_back(Token {
                span,
                token_type: FlowMappingEnd,
            })
        }
    }

    fn fetch_plain_scalar(&mut self) -> ScanResult {
        self.save_simple_key();
        self.simple_key_allowed = false;

        let tok = self.scan_plain_scalar()?;

        self.tokens.push_back(tok);
        Ok(())
    }

    fn fetch_flow_entry(&mut self) -> ScanResult {
        self.remove_simple_key()?;
        self.simple_key_allowed = true;

        self.end_implicit_mapping(self.mark);

        let start_mark = self.mark;
        self.skip_non_blank();
        self.skip_ws_to_eol(SkipTabs::Yes)?;

        let span = self.get_span(start_mark);
        self.tokens.push_back(Token {
            token_type: TokenType::FlowEntry,
            span,
        });
        Ok(())
    }

    fn fetch_block_entry(&mut self) -> ScanResult {
        if self.flow_level > 0 {
            // - * only allowed in block
            return Err(YamlError::new_str(
                self.mark,
                r#""-" is only valid inside a block"#,
            ));
        }
        // Check if we are allowed to start a new entry.
        if !self.simple_key_allowed {
            return Err(YamlError::new_str(
                self.mark,
                "block sequence entries are not allowed in this context",
            ));
        }

        // ???, fixes test G9HC.
        if let Some(Token {
            span,
            token_type: TokenType::Anchor(..) | TokenType::Tag { .. },
        }) = self.tokens.back()
            && self.mark.col == 1
            && span.start.col == 1
            && self.indent > 0
        {
            return Err(YamlError::new_str(
                self.mark,
                "block sequence entries are not allowed in this context",
            ));
        }

        // Skip over the `-`.
        let mark = self.mark;
        self.skip_non_blank();

        // generate BLOCK-SEQUENCE-START if indented
        self.roll_indent(mark.col, None, TokenType::BlockSequenceStart, mark);
        let found_tabs = self.skip_ws_to_eol(SkipTabs::Yes)?.found_tabs();
        if found_tabs && self.src.next_byte_is(b'-') && is_blank_or_break(self.src.peek_nth(1)) {
            return Err(YamlError::new_str(
                self.mark,
                "'-' must be followed by a valid YAML whitespace",
            ));
        }

        self.skip_ws_to_eol(SkipTabs::No)?;
        // ? self.input.lookahead(1);
        if self.src.next_is_break() || self.src.next_is_flow() {
            self.roll_one_col_indent();
        }

        self.remove_simple_key()?;
        self.simple_key_allowed = true;

        let span = self.get_span(mark);
        self.tokens.push_back(Token {
            span,
            token_type: TokenType::BlockEntry,
        });

        Ok(())
    }

    fn fetch_key(&mut self) -> ScanResult {
        let start_mark = self.mark;
        if self.flow_level == 0 {
            // Check if we are allowed to start a new key (not necessarily simple).
            if !self.simple_key_allowed {
                return Err(YamlError::new_str(
                    self.mark,
                    "mapping keys are not allowed in this context",
                ));
            }
            self.roll_indent(
                start_mark.col,
                None,
                TokenType::BlockMappingStart,
                start_mark,
            );
        } else {
            // The scanner, upon emitting a `Key`, will prepend a `MappingStart` event.
            self.flow_mapping_started = true;
        }

        self.remove_simple_key()?;

        self.simple_key_allowed = self.flow_level == 0;

        self.skip_non_blank();
        self.skip_yaml_whitespace()?;
        if self.src.peek() == b'\t' {
            return Err(YamlError::new_str(
                self.mark,
                "tabs disallowed in this context",
            ));
        }
        let span = self.get_span(start_mark);
        self.tokens.push_back(Token {
            span,
            token_type: TokenType::Key,
        });
        Ok(())
    }

    fn skip_yaml_whitespace(&mut self) -> ScanResult {
        let mut need_whitespace = true;
        loop {
            match self.src.peek() {
                b' ' => {
                    self.skip_blank();

                    need_whitespace = false;
                }
                b'\n' | b'\r' => {
                    // ? self.src.lookahead(2);
                    self.skip_linebreak();
                    if self.flow_level == 0 {
                        self.simple_key_allowed = true;
                    }
                    need_whitespace = false;
                }
                b'#' => {
                    let comment_length = self.src.skip_while_non_breakz();
                    self.mark.pos += comment_length;
                    self.mark.col = self.mark.col.saturating_add(comment_length as u32);
                }
                _ => break,
            }
        }

        if need_whitespace {
            Err(YamlError::new_str(self.mark, "expected whitespace"))
        } else {
            Ok(())
        }
    }

    fn fetch_value(&mut self) -> ScanResult {
        let sk = self.simple_keys.last().unwrap().clone();
        let start_mark = self.mark;
        let is_implicit_flow_mapping =
            !self.implicit_flow_mapping_states.is_empty() && !self.flow_mapping_started;
        if is_implicit_flow_mapping {
            *self.implicit_flow_mapping_states.last_mut().unwrap() = ImplicitMappingState::Inside;
        }

        // Skip over ':'.
        self.skip_non_blank();
        if self.src.peek() == b'\t'
            && !self.skip_ws_to_eol(SkipTabs::Yes)?.has_valid_yaml_ws()
            && (self.src.peek() == b'-' || self.src.next_is_alpha())
        {
            return Err(YamlError::new_str(
                self.mark,
                "':' must be followed by a valid YAML whitespace",
            ));
        }

        if sk.possible {
            // insert simple key
            let tok = Token {
                span: Span::empty(sk.mark),
                token_type: TokenType::Key,
            };
            self.insert_token(sk.token_number - self.tokens_parsed, tok);
            if is_implicit_flow_mapping {
                if sk.mark.line < start_mark.line {
                    return Err(YamlError::new_str(
                        start_mark,
                        "illegal placement of ':' indicator",
                    ));
                }
                self.insert_token(
                    sk.token_number - self.tokens_parsed,
                    Token {
                        span: Span::empty(sk.mark),
                        token_type: FlowMappingStart,
                    },
                );
            }

            // Add the BLOCK-MAPPING-START token if needed.
            self.roll_indent(
                sk.mark.col,
                Some(sk.token_number),
                TokenType::BlockMappingStart,
                sk.mark,
            );
            self.roll_one_col_indent();

            self.simple_keys.last_mut().unwrap().possible = false;
            self.simple_key_allowed = false;
        } else {
            if is_implicit_flow_mapping {
                self.tokens.push_back(Token {
                    span: Span::empty(start_mark),
                    token_type: FlowMappingStart,
                });
            }
            // The ':' indicator follows a complex key.
            if self.flow_level == 0 {
                if !self.simple_key_allowed {
                    return Err(YamlError::new_str(
                        start_mark,
                        "mapping values are not allowed in this context",
                    ));
                }

                self.roll_indent(
                    start_mark.col,
                    None,
                    TokenType::BlockMappingStart,
                    start_mark,
                );
            }
            self.roll_one_col_indent();

            self.simple_key_allowed = self.flow_level == 0;
        }
        self.tokens.push_back(Token {
            span: Span::empty(start_mark),
            token_type: TokenType::Value,
        });

        Ok(())
    }

    fn fetch_flow_value(&mut self) -> ScanResult {
        let nc = self.src.peek_nth(1);

        // If we encounter a ':' inside a flow collection and it is not immediately
        // followed by a blank or breakz:
        //   - We must check whether an adjacent value is allowed
        //     `["a":[]]` is valid. If the key is double-quoted, no need for a space. This
        //     is needed for JSON compatibility.
        //   - If not, we must ensure there is a space after the ':' and before its value.
        //     `[a: []]` is valid while `[a:[]]` isn't. `[a:b]` is treated as `["a:b"]`.
        //   - But if the value is empty (null), then it's okay.
        // The last line is for YAMLs like `[a:]`. The ':' is followed by a ']' (which is a
        // flow character), but the ']' is not the value. The value is an invisible empty
        // space which is represented as null ('~').
        if self.mark.pos != self.adjacent_value_allowed_at && matches!(nc, b'[' | b'{') {
            return Err(YamlError::new_str(
                self.mark,
                "':' may not precede any of `[{` in flow mapping",
            ));
        }

        self.fetch_value()
    }

    fn fetch_anchor(&mut self, is_alias: bool) -> ScanResult {
        self.save_simple_key();
        self.simple_key_allowed = false;

        let tok = self.scan_anchor(is_alias)?;

        self.tokens.push_back(tok);

        Ok(())
    }

    fn fetch_tag(&mut self) -> ScanResult {
        self.save_simple_key();
        self.simple_key_allowed = false;

        let tok = self.scan_tag()?;
        self.tokens.push_back(tok);
        Ok(())
    }

    fn fetch_block_scalar(&mut self, is_literal: bool) -> ScanResult {
        self.save_simple_key();
        self.simple_key_allowed = true;
        let tok = self.scan_block_scalar(is_literal)?;

        self.tokens.push_back(tok);
        Ok(())
    }

    fn fetch_flow_scalar(&mut self, single: bool) -> ScanResult {
        self.save_simple_key();
        self.simple_key_allowed = false;

        let tok = self.scan_flow_scalar(single)?;

        // From spec: To ensure JSON compatibility, if a key inside a flow mapping is JSON-like,
        // YAML allows the following value to be specified adjacent to the “:”.
        self.skip_to_next_token()?;
        self.adjacent_value_allowed_at = self.mark.pos;

        self.tokens.push_back(tok);
        Ok(())
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
            Err(YamlError::new_str(
                self.mark,
                "Invalid content after document end marker",
            ))
        } else {
            Ok(())
        }
    }

    fn skip_n_non_blank(&mut self, count: usize) {
        self.src.skip(count);

        self.mark.pos += count;
        self.mark.col += count as u32;
        self.leading_whitespace = false;
    }

    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> Result<SkipTabs, YamlError> {
        let (n_bytes, result) = self.src.skip_ws_to_eol(skip_tabs);

        self.mark.col += n_bytes;
        self.mark.pos += n_bytes as usize;
        result.map_err(|message| YamlError::new_str(self.mark, message))
    }

    #[inline]
    fn skip_linebreak(&mut self) {
        match self.src.peek_two() {
            [b'\r', b'\n'] => {
                self.mark.pos += 2;
                self.mark.col = 1;
                self.mark.line += 1;
                self.leading_whitespace = true;
                self.src.skip(2);
            }
            [b'\n', _] => {
                self.mark.pos += 1;
                self.mark.col = 1;
                self.mark.line += 1;
                self.leading_whitespace = true;
                self.src.skip(1);
            }
            _ => {}
        }
    }

    fn skip_blank(&mut self) {
        self.src.skip(1);

        self.mark.pos += 1;
        self.mark.col += 1;
    }

    fn skip_non_blank(&mut self) {
        self.src.skip(1);

        self.mark.pos += 1;
        self.mark.col += 1;
        self.leading_whitespace = false;
    }

    fn is_within_block(&self) -> bool {
        !self.indents.is_empty()
    }

    fn skip_to_next_token(&mut self) -> ScanResult {
        loop {
            match self.src.peek() {
                // Tabs may not be used as indentation.
                // "Indentation" only exists as long as a block is started, but does not exist
                // inside of flow-style constructs. Tabs are allowed as part of leading
                // whitespaces outside of indentation.
                // If a flow-style construct is in an indented block, its contents must still be
                // indented. Also, tabs are allowed anywhere in it if it has no content.
                b'\t'
                    if self.is_within_block()
                        && self.leading_whitespace
                        && self.mark.col < self.indent =>
                {
                    self.skip_ws_to_eol(SkipTabs::Yes)?;
                    // If we have content on that line with a tab, return an error.
                    if !self.src.next_is_breakz() {
                        return Err(YamlError::new_str(
                            self.mark,
                            "tabs disallowed within this context (block indentation)",
                        ));
                    }
                }
                b'\t' | b' ' => self.skip_blank(),
                b'\n' | b'\r' => {
                    // ? self.src.lookahead(2);
                    self.skip_linebreak();
                    if self.flow_level == 0 {
                        self.simple_key_allowed = true;
                    }
                }
                b'#' => {
                    let comment_length = self.src.skip_while_non_breakz();
                    self.mark.pos += comment_length;
                    self.mark.col += comment_length as u32;
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn scan_directive(&mut self) -> YamlResult<Token<'input>> {
        let start_mark = self.mark;
        self.skip_non_blank();

        let name = self.scan_directive_name()?;
        let tok = match &name[..] {
            b"YAML" => self.scan_version_directive_value(&start_mark)?,
            b"TAG" => self.scan_tag_directive_value(&start_mark)?,
            // XXX This should be a warning instead of an error
            _ => {
                // skip current line
                let line_len = self.src.skip_while_non_breakz();
                self.mark.pos += line_len;
                self.mark.col += line_len as u32;
                // XXX return an empty TagDirective token
                Token {
                    span: Span::new(start_mark, self.mark),
                    token_type: TokenType::TagDirective {
                        handle: Cow::default(),
                        prefix: Cow::default(),
                    },
                }
                // return Err(ScanError::new_str(start_mark,
                //     "while scanning a directive, found unknown directive name"))
            }
        };

        self.skip_ws_to_eol(SkipTabs::Yes)?;

        if self.src.next_is_break() {
            // self.src.lookahead(2);
            self.skip_linebreak();
            Ok(tok)
        } else {
            Err(YamlError::new_str(
                start_mark,
                "while scanning a directive, did not find expected comment or line break",
            ))
        }
    }

    #[allow(clippy::too_many_lines)]
    fn scan_plain_scalar(&mut self) -> Result<Token<'input>, YamlError> {
        self.unroll_non_block_indents();
        let indent = self.indent + 1;
        let start_mark = self.mark;

        if self.flow_level > 0 && start_mark.col < indent {
            return Err(YamlError::new_str(
                start_mark,
                "invalid indentation in flow construct",
            ));
        }

        let mut string: Vec<u8> = Vec::with_capacity(32);
        self.buf_whitespaces.clear();
        self.buf_leading_break.clear();
        self.buf_trailing_breaks.clear();
        let mut end_mark = self.mark;

        loop {
            // ? self.input.lookahead(4);
            let next_is_document_indicator = self.src.next_is_document_indicator();
            if (self.leading_whitespace && next_is_document_indicator) || self.src.peek() == b'#' {
                break;
            }

            if self.flow_level > 0 && self.src.peek() == b'-' && is_flow(self.src.peek_nth(1)) {
                return Err(YamlError::new_str(
                    self.mark,
                    "plain scalar cannot start with '-' followed by ,[]{}",
                ));
            }

            if !self.src.next_is_blank_or_breakz()
                && self.src.next_can_be_plain_scalar(self.flow_level > 0)
            {
                if self.leading_whitespace {
                    if self.buf_leading_break.is_empty() {
                        string.extend_from_slice(&self.buf_leading_break);
                        string.extend_from_slice(&self.buf_trailing_breaks);
                        self.buf_trailing_breaks.clear();
                        self.buf_leading_break.clear();
                    } else {
                        if self.buf_trailing_breaks.is_empty() {
                            string.push(b' ');
                        } else {
                            string.extend_from_slice(&self.buf_trailing_breaks);
                            self.buf_trailing_breaks.clear();
                        }
                        self.buf_leading_break.clear();
                    }
                    self.leading_whitespace = false;
                } else if !self.buf_whitespaces.is_empty() {
                    string.extend_from_slice(&self.buf_whitespaces);
                    self.buf_whitespaces.clear();
                }

                // We can unroll the first iteration of the loop.
                string.push(self.src.peek());
                self.skip_non_blank();
                string.reserve(self.src.bufmaxlen());

                // Add content non-blank characters to the scalar.
                let mut end = false;
                while !end {
                    // Fill the buffer once and process all characters in the buffer until the next
                    // fetch. Note that `next_can_be_plain_scalar` needs 2 lookahead characters,
                    // hence the `for` loop looping `self.input.bufmaxlen() - 1` times.
                    // ? self.src.lookahead(self.src.bufmaxlen());
                    for _ in 0..self.src.bufmaxlen() - 1 {
                        if self.src.next_is_blank_or_breakz()
                            || !self.src.next_can_be_plain_scalar(self.flow_level > 0)
                        {
                            end = true;
                            break;
                        }
                        string.push(self.src.peek());
                        self.skip_non_blank();
                    }
                }
                end_mark = self.mark;
            }

            // We may reach the end of a plain scalar if:
            //  - We reach eof
            //  - We reach ": "
            //  - We find a flow character in a flow context
            if !(self.src.next_is_blank() || self.src.next_is_break()) {
                break;
            }

            // Process blank characters.
            // ? self.input.lookahead(2);
            while self.src.next_is_blank_or_break() {
                if self.src.next_is_blank() {
                    if !self.leading_whitespace {
                        self.buf_whitespaces.push(self.src.peek());
                        self.skip_blank();
                    } else if self.mark.col < indent && self.src.peek() == b'\t' {
                        // Tabs in an indentation columns are allowed if and only if the line is
                        // empty. Skip to the end of the line.
                        self.skip_ws_to_eol(SkipTabs::Yes)?;
                        if !self.src.next_is_breakz() {
                            return Err(YamlError::new_str(
                                start_mark,
                                "while scanning a plain scalar, found a tab",
                            ));
                        }
                    } else {
                        self.skip_blank();
                    }
                } else {
                    // Check if it is a first line break
                    if self.leading_whitespace {
                        // TODO check this works
                        self.skip_linebreak();
                        self.buf_trailing_breaks.push(b'\n');
                    } else {
                        self.buf_whitespaces.clear();
                        self.skip_linebreak();
                        self.buf_leading_break.push(b'\n');
                        self.leading_whitespace = true;
                    }
                }
                // ? self.src.lookahead(2);
            }

            // check indentation level
            if self.flow_level == 0 && self.mark.col < indent {
                break;
            }
        }

        if self.leading_whitespace {
            self.simple_key_allowed = true;
        }

        if string.is_empty() {
            // `fetch_plain_scalar` must absolutely consume at least one byte. Otherwise,
            // `fetch_next_token` will never stop calling it. An empty plain scalar may happen with
            // erroneous inputs such as "{...".
            Err(YamlError::new_str(
                start_mark,
                "unexpected end of plain scalar",
            ))
        } else {
            Ok(Token {
                span: Span::new(start_mark, end_mark),
                token_type: TokenType::Scalar {
                    scalar_type: Plain,
                    value: unsafe { Cow::Owned(String::from_utf8_unchecked(string)) },
                },
            })
        }
    }

    #[allow(clippy::too_many_lines)]
    fn scan_flow_scalar(&mut self, single: bool) -> Result<Token<'input>, YamlError> {
        let start_mark = self.mark;

        let mut string = Vec::new();
        let mut leading_break = Vec::new();
        let mut trailing_breaks = Vec::new();
        let mut whitespaces = Vec::new();
        let mut leading_blanks;

        /* Eat the left quote. */
        self.skip_non_blank();

        loop {
            /* Check for a document indicator. */
            // ? self.src.lookahead(4);

            if self.mark.col == 1 && self.src.next_is_document_indicator() {
                return Err(YamlError::new_str(
                    start_mark,
                    "while scanning a quoted scalar, found unexpected document indicator",
                ));
            }

            if self.src.next_is_z() {
                return Err(YamlError::new_str(
                    start_mark,
                    "while scanning a quoted scalar, found unexpected end of stream",
                ));
            }

            if self.mark.col < self.indent {
                return Err(YamlError::new_str(
                    start_mark,
                    "invalid indentation in quoted scalar",
                ));
            }

            leading_blanks = false;
            self.consume_flow_scalar_non_whitespace_chars(
                single,
                &mut string,
                &mut leading_blanks,
                &start_mark,
            )?;

            match self.src.peek() {
                b'\'' if single => break,
                b'"' if !single => break,
                _ => {}
            }

            // Consume blank characters.
            while self.src.next_is_blank() || self.src.next_is_break() {
                if self.src.next_is_blank() {
                    // Consume a space or a tab character.
                    if leading_blanks {
                        if self.src.peek() == b'\t' && self.mark.col < self.indent {
                            return Err(YamlError::new_str(
                                self.mark,
                                "tab cannot be used as indentation",
                            ));
                        }
                        self.skip_blank();
                    } else {
                        whitespaces.push(self.src.peek());
                        self.skip_blank();
                    }
                } else {
                    // ? self.src.lookahead(2);
                    // Check if it is a first line break.
                    if leading_blanks {
                        self.read_break(&mut trailing_breaks);
                    } else {
                        whitespaces.clear();
                        self.read_break(&mut leading_break);
                        leading_blanks = true;
                    }
                }
                // ? self.input.lookahead(1);
            }

            // Join the whitespaces or fold line breaks.
            if leading_blanks {
                if leading_break.is_empty() {
                    string.extend_from_slice(&leading_break);
                    string.extend_from_slice(&trailing_breaks);
                    trailing_breaks.clear();
                    leading_break.clear();
                } else {
                    if trailing_breaks.is_empty() {
                        string.push(b' ');
                    } else {
                        string.extend_from_slice(&trailing_breaks);
                        trailing_breaks.clear();
                    }
                    leading_break.clear();
                }
            } else {
                string.extend_from_slice(&whitespaces);
                whitespaces.clear();
            }
        } // loop

        // Eat the right quote.
        self.skip_non_blank();
        // Ensure there is no invalid trailing content.
        self.skip_ws_to_eol(SkipTabs::Yes)?;
        match self.src.peek() {
            // These can be encountered in flow sequences or mappings.
            b',' | b'}' | b']' if self.flow_level > 0 => {}
            // An end-of-line / end-of-stream is fine. No trailing content.
            c if is_breakz(c) => {}
            // ':' can be encountered if our scalar is a key.
            // Outside of flow contexts, keys cannot span multiple lines
            b':' if self.flow_level == 0 && start_mark.line == self.mark.line => {}
            // Inside a flow context, this is allowed.
            b':' if self.flow_level > 0 => {}
            _ => {
                return Err(YamlError::new_str(
                    self.mark,
                    "invalid trailing content after double-quoted scalar",
                ));
            }
        }

        let style = if single {
            ScalarType::SingleQuote
        } else {
            ScalarType::DoubleQuote
        };
        Ok(Token {
            span: Span::new(start_mark, self.mark),
            token_type: TokenType::Scalar {
                scalar_type: style,
                value: unsafe { Cow::Owned(String::from_utf8_unchecked(string)) },
            },
        })
    }

    fn scan_block_scalar(&mut self, literal: bool) -> Result<Token<'input>, YamlError> {
        let start_mark = self.mark;
        let mut chomping = ChompIndicator::Clip;
        let mut increment: usize = 0;
        let mut indent: u32 = 0;
        let mut trailing_blank: bool;
        let mut leading_blank: bool = false;
        let scalar_type = if literal {
            ScalarType::Literal
        } else {
            ScalarType::Folded
        };

        let mut string = Vec::<u8>::new();
        let mut leading_break = Vec::<u8>::new();
        let mut trailing_breaks = Vec::<u8>::new();
        let mut chomping_break = Vec::<u8>::new();

        // skip '|' or '>'
        self.skip_non_blank();
        self.unroll_non_block_indents();

        if self.src.peek() == b'+' || self.src.peek() == b'-' {
            if self.src.peek() == b'+' {
                chomping = ChompIndicator::Keep;
            } else {
                chomping = ChompIndicator::Strip;
            }
            self.skip_non_blank();
            // ? self.src.lookahead(1);
            if self.src.peek().is_ascii_digit() {
                if self.src.peek() == b'0' {
                    return Err(YamlError::new_str(
                        start_mark,
                        "while scanning a block scalar, found an indentation indicator equal to 0",
                    ));
                }
                increment = (self.src.peek() - b'0') as usize;
                self.skip_non_blank();
            }
        } else if self.src.peek().is_ascii_digit() {
            if self.src.peek() == b'0' {
                return Err(YamlError::new_str(
                    start_mark,
                    "while scanning a block scalar, found an indentation indicator equal to 0",
                ));
            }

            increment = (self.src.peek() - b'0') as usize;
            self.skip_non_blank();
            // ? self.src.lookahead(1);
            if self.src.peek() == b'+' || self.src.peek() == b'-' {
                if self.src.peek() == b'+' {
                    chomping = ChompIndicator::Keep;
                } else {
                    chomping = ChompIndicator::Strip;
                }
                self.skip_non_blank();
            }
        }

        self.skip_ws_to_eol(SkipTabs::Yes)?;

        // Check if we are at the end of the line.
        // self.input.lookahead(1);
        if !self.src.next_is_breakz() {
            return Err(YamlError::new_str(
                start_mark,
                "while scanning a block scalar, did not find expected comment or line break",
            ));
        }

        if self.src.next_is_break() {
            // self.src.lookahead(2);
            self.read_break(&mut chomping_break);
        }

        if self.src.peek() == b'\t' {
            return Err(YamlError::new_str(
                start_mark,
                "a block scalar content cannot start with a tab",
            ));
        }

        if increment > 0 {
            indent = if self.indent >= 1 {
                self.indent + increment as u32
            } else {
                increment as u32
            }
        }

        // Scan the leading line breaks and determine the indentation level if needed.
        if indent == 0 {
            self.skip_block_scalar_first_line_indent(&mut indent, &mut trailing_breaks);
        } else {
            self.skip_block_scalar_indent(indent, &mut trailing_breaks);
        }

        // We have an end-of-stream with no content, e.g.:
        // ```yaml
        // - |+
        // ```
        if self.src.next_is_z() {
            let contents = match chomping {
                // We strip trailing linebreaks. Nothing remain.
                ChompIndicator::Strip => Vec::new(),
                // There was no newline after the chomping indicator.
                _ if self.mark.line == start_mark.line => Vec::new(),
                // We clip lines, and there was a newline after the chomping indicator.
                // All other breaks are ignored.
                ChompIndicator::Clip => chomping_break,
                // We keep lines. There was a newline after the chomping indicator but nothing
                // else.
                ChompIndicator::Keep if trailing_breaks.is_empty() => chomping_break,
                // Otherwise, the newline after chomping is ignored.
                ChompIndicator::Keep => trailing_breaks,
            };
            return Ok(Token {
                span: self.get_span(start_mark),
                token_type: TokenType::Scalar {
                    scalar_type,
                    value: unsafe { Cow::Owned(String::from_utf8_unchecked(contents)) },
                },
            });
        }

        if self.mark.col < indent && self.mark.col > self.indent {
            return Err(YamlError::new_str(
                self.mark,
                "wrongly indented line in block scalar",
            ));
        }

        let mut line_buffer = Vec::with_capacity(100);
        let start_mark = self.mark;
        while self.mark.col == indent && !self.src.next_is_z() {
            if indent == 1 {
                // self.src.lookahead(4);
                if self.next_is_document_end() {
                    break;
                }
            }

            // We are at the first content character of a content line.
            trailing_blank = self.src.next_is_blank();
            if !literal && !leading_break.is_empty() && !leading_blank && !trailing_blank {
                string.extend_from_slice(&trailing_breaks);
                if trailing_breaks.is_empty() {
                    string.push(b' ');
                }
            } else {
                string.extend_from_slice(&leading_break);
                string.extend_from_slice(&trailing_breaks);
            }

            leading_break.clear();
            trailing_breaks.clear();

            leading_blank = self.src.next_is_blank();

            self.scan_block_scalar_content_line(&mut string, &mut line_buffer);

            // break on EOF
            // ? self.input.lookahead(2);
            if self.src.next_is_z() {
                break;
            }

            self.read_break(&mut leading_break);

            // Eat the following indentation spaces and line breaks.
            self.skip_block_scalar_indent(indent, &mut trailing_breaks);
        }

        // Chomp the tail.
        if chomping != ChompIndicator::Strip {
            string.extend_from_slice(&leading_break);
            // If we had reached an eof but the last character wasn't an end-of-line, check if the
            // last line was indented at least as the rest of the scalar, then we need to consider
            // there is a newline.
            let is_greater_col = self.mark.col > indent.max(1);
            if self.src.next_is_z() && is_greater_col {
                string.push(b'\n');
            }
        }

        if chomping == ChompIndicator::Keep {
            string.extend_from_slice(&trailing_breaks);
        }

        Ok(Token {
            span: Span::new(start_mark, self.mark),
            token_type: TokenType::Scalar {
                scalar_type,
                value: Cow::Owned(unsafe { String::from_utf8_unchecked(string) }),
            },
        })
    }

    fn scan_block_scalar_content_line(&mut self, string: &mut Vec<u8>, line_buffer: &mut Vec<u8>) {
        // Start by evaluating characters in the buffer.
        while !self.src.buf_is_empty() && !self.src.next_is_break() {
            string.push(self.src.peek());
            // We may technically skip non-blank characters. However, the only distinction is
            // to determine what is leading whitespace and what is not. Here, we read the
            // contents of the line until either eof or a linebreak. We know we will not read
            // `self.leading_whitespace` until the end of the line, where it will be reset.
            // This allows us to call a slightly less expensive function.
            self.skip_blank();
        }

        // All characters that were in the buffer were consumed. We need to check if more
        // follow.
        if self.src.buf_is_empty() {
            // We will read all consecutive non-breakz characters. We push them into a
            // temporary buffer. The main difference with going through `self.buffer` is that
            // characters are appended here as their real size (1B for ascii, or up to 4 bytes for
            // UTF-8). We can then use the internal `line_buffer` `Vec` to push data into `string`
            // (using `String::push_str`).
            line_buffer.extend_from_slice(self.src.raw_read_non_breakz_ch());

            // We need to manually update our position; we haven't called a `skip` function.
            let n_chars = line_buffer.len();
            self.mark.col += n_chars as u32;
            self.mark.pos += n_chars;

            // We can now append our bytes to our `string`.
            string.reserve(line_buffer.len());
            string.extend_from_slice(line_buffer);
            // This clears the _contents_ without touching the _capacity_.
            line_buffer.clear();
        }
    }

    fn scan_anchor(&mut self, alias: bool) -> Result<Token<'input>, YamlError> {
        let mut string = Vec::new();
        let start_mark = self.mark;

        self.skip_non_blank();
        while is_anchor_char(self.src.peek()) {
            string.push(self.src.peek());
            self.skip_non_blank();
        }

        if string.is_empty() {
            return Err(YamlError::new_str(
                start_mark,
                "while scanning an anchor or alias, did not find expected alphabetic or numeric character",
            ));
        }

        let tok = if alias {
            TokenType::Alias(Cow::Owned(unsafe { String::from_utf8_unchecked(string) }))
        } else {
            TokenType::Anchor(Cow::Owned(unsafe { String::from_utf8_unchecked(string) }))
        };
        Ok(Token {
            span: Span::new(start_mark, self.mark),
            token_type: tok,
        })
    }

    fn scan_tag(&mut self) -> Result<Token<'input>, YamlError> {
        let start_mark = self.mark;
        let mut handle = Vec::new();
        let mut suffix;

        // Check if the tag is in the canonical form (verbatim).
        // self.input.lookahead(2);

        if self.src.nth_byte_is(1, b'<') {
            suffix = self.scan_verbatim_tag(&start_mark)?;
        } else {
            // The tag has either the '!suffix' or the '!handle!suffix'
            handle = self.scan_tag_handle(false, &start_mark)?;
            // Check if it is, indeed, handle.
            if handle.len() >= 2 && handle.starts_with(b"!") && handle.ends_with(b"!") {
                // A tag handle starting with "!!" is a secondary tag handle.
                let is_secondary_handle = handle == b"!!";
                suffix = self.scan_tag_shorthand_suffix(
                    false,
                    is_secondary_handle,
                    &b"".to_vec(),
                    &start_mark,
                )?;
            } else {
                suffix = self.scan_tag_shorthand_suffix(false, false, &handle, &start_mark)?;

                handle = b"!".to_vec();
                // A special case: the '!' tag.  Set the handle to '' and the
                // suffix to '!'.
                if suffix.is_empty() {
                    handle.clear();
                    suffix.push(b'!');
                }
            }
        }

        if is_blank_or_breakz(self.src.peek()) || (self.flow_level > 0 && self.src.next_is_flow()) {
            // XXX: ex 7.2, an empty scalar can follow a secondary tag
            Ok(Token {
                span: Span::new(start_mark, self.mark),
                // SAFETY: handle and prefix must contain valid Vec<u8>
                token_type: unsafe { TokenType::new_tag_unchecked(handle, suffix) },
            })
        } else {
            Err(YamlError::new_str(
                start_mark,
                "while scanning a tag, did not find expected whitespace or line break",
            ))
        }
    }

    fn scan_verbatim_tag(&mut self, start_mark: &Marker) -> Result<Vec<u8>, YamlError> {
        // Eat `!<`
        self.skip_non_blank();
        self.skip_non_blank();

        let mut string = Vec::new();
        while is_uri_char(self.src.peek()) {
            if self.src.peek() == b'%' {
                string.extend(self.scan_uri_escapes(start_mark)?);
            } else {
                string.push(self.src.peek());
                self.skip_non_blank();
            }
        }

        if self.src.peek() != b'>' {
            return Err(YamlError::new_str(
                *start_mark,
                "while scanning a verbatim tag, did not find the expected '>'",
            ));
        }
        self.skip_non_blank();

        Ok(string)
    }

    fn scan_tag_handle(&mut self, directive: bool, mark: &Marker) -> Result<Vec<u8>, YamlError> {
        let mut string = Vec::new();
        if self.src.peek() != b'!' {
            return Err(YamlError::new_str(
                *mark,
                "while scanning a tag, did not find expected '!'",
            ));
        }

        string.push(self.src.peek());
        self.skip_non_blank();

        let n_chars = self.src.fetch_while_is_alpha(&mut string);
        self.mark.pos += n_chars;
        self.mark.col += n_chars as u32;

        // Check if the trailing character is '!' and copy it.
        if self.src.peek() == b'!' {
            string.push(self.src.peek());
            self.skip_non_blank();
        } else if directive && string != b"!" {
            // It's either the '!' tag or not really a tag handle.  If it's a %TAG
            // directive, it's an error.  If it's a tag token, it must be a part of
            // URI.
            return Err(YamlError::new_str(
                *mark,
                "while parsing a tag directive, did not find expected '!'",
            ));
        }
        Ok(string)
    }

    fn scan_tag_shorthand_suffix(
        &mut self,
        _directive: bool,
        _is_secondary: bool,
        head: &Vec<u8>,
        mark: &Marker,
    ) -> Result<Vec<u8>, YamlError> {
        let mut length = head.len();
        let mut string = Vec::new();

        // Copy the head if needed.
        // Note that we don't copy the leading '!' character.
        if length > 1 {
            string.extend_from_slice(&head[1..]);
        }

        while is_tag_char(self.src.peek()) {
            // Check if it is a URI-escape sequence.
            if self.src.peek() == b'%' {
                string.extend_from_slice(&self.scan_uri_escapes(mark)?);
            } else {
                string.push(self.src.peek());
                self.skip_non_blank();
            }

            length += 1;
        }

        if length == 0 {
            return Err(YamlError::new_str(
                *mark,
                "while parsing a tag, did not find expected tag URI",
            ));
        }

        Ok(string)
    }

    fn skip_block_scalar_first_line_indent(&mut self, indent: &mut u32, breaks: &mut Vec<u8>) {
        let mut max_indent = 0;
        loop {
            // Consume all spaces. Tabs cannot be used as indentation.
            while self.src.peek() == b' ' {
                self.skip_blank();
            }

            if self.mark.col > max_indent {
                max_indent = self.mark.col;
            }

            if self.src.next_is_break() {
                // If our current line is empty, skip over the break and continue looping.
                // self.src.lookahead(2);
                self.read_break(breaks);
            } else {
                // Otherwise, we have a content line. Return control.
                break;
            }
        }

        // In case a yaml looks like:
        // ```yaml
        // |
        // foo
        // bar
        // ```
        // We need to set the indent to 0 and not 1. In all other cases, the indent must be at
        // least 1. When in the above example, `self.indent` will be set to -1.
        *indent = max_indent.max(self.indent + 1);
        if self.indent > 0 {
            *indent = (*indent).max(1);
        }
    }

    /// Skip the block scalar indentation and empty lines.
    fn skip_block_scalar_indent(&mut self, indent: u32, breaks: &mut Vec<u8>) {
        loop {
            // Consume all spaces. Tabs cannot be used as indentation.
            if (indent as usize) < self.src.bufmaxlen() - 2 {
                // ? self.src.lookahead(self.input.bufmaxlen());
                while self.mark.col < indent && self.src.peek() == b' ' {
                    self.skip_blank();
                }
            } else {
                loop {
                    // ? self.input.lookahead(self.input.bufmaxlen());
                    while !self.src.buf_is_empty()
                        && self.mark.col < indent
                        && self.src.peek() == b' '
                    {
                        self.skip_blank();
                    }
                    // If we reached our indent, we can break. We must also break if we have
                    // reached content or EOF; that is, the buffer is not empty and the next
                    // character is not a space.
                    if self.mark.col == indent
                        || (!self.src.buf_is_empty() && self.src.peek() != b' ')
                    {
                        break;
                    }
                }
                // ? self.input.lookahead(2);
            }

            // If our current line is empty, skip over the break and continue looping.
            if self.src.next_is_break() {
                self.read_break(breaks);
            } else {
                // Otherwise, we have a content line. Return control.
                break;
            }
        }
    }

    #[inline]
    fn read_break(&mut self, s: &mut Vec<u8>) {
        self.skip_linebreak();
        s.push(b'\n');
    }

    fn consume_flow_scalar_non_whitespace_chars(
        &mut self,
        single: bool,
        string: &mut Vec<u8>,
        leading_blanks: &mut bool,
        start_mark: &Marker,
    ) -> Result<(), YamlError> {
        // ? self.input.lookahead(2);
        while !is_blank_or_breakz(self.src.peek()) {
            match self.src.peek() {
                // Check for an escaped single quote.
                b'\'' if self.src.peek_nth(1) == b'\'' && single => {
                    string.push(b'\'');
                    self.skip_n_non_blank(2);
                }
                // Check for the right quote.
                b'\'' if single => break,
                b'"' if !single => break,
                // Check for an escaped line break.
                b'\\' if !single && is_break(self.src.peek_nth(1)) => {
                    // ? self.input.lookahead(3);
                    self.skip_non_blank();
                    self.skip_linebreak();
                    *leading_blanks = true;
                    break;
                }
                // Check for an escape sequence.
                b'\\' if !single => {
                    let chr = self.resolve_flow_scalar_escape_sequence(start_mark)?;
                    string.extend_from_slice(chr.to_string().as_bytes());
                }
                c => {
                    string.push(c);
                    self.skip_non_blank();
                }
            }
            // ? self.input.lookahead(2);
        }
        Ok(())
    }

    /// Escape the sequence we encounter in a flow scalar.
    ///
    /// `self.input.peek()` must point to the `\` starting the escape sequence.
    ///
    /// # Errors
    /// Return an error if an invalid escape sequence is found.
    fn resolve_flow_scalar_escape_sequence(
        &mut self,
        start_mark: &Marker,
    ) -> Result<char, YamlError> {
        let mut code_length = 0usize;
        let mut ret = '\0';

        match self.src.peek_nth(1) {
            b'0' => ret = '\0',
            b'a' => ret = '\x07',
            b'b' => ret = '\x08',
            b't' | b'\t' => ret = '\t',
            b'n' => ret = '\n',
            b'v' => ret = '\x0b',
            b'f' => ret = '\x0c',
            b'r' => ret = '\x0d',
            b'e' => ret = '\x1b',
            b' ' => ret = '\x20',
            b'"' => ret = '"',
            b'/' => ret = '/',
            b'\\' => ret = '\\',
            // Unicode next line (#x85)
            b'N' => ret = char::from_u32(0x85).unwrap(),
            // Unicode non-breaking space (#xA0)
            b'_' => ret = char::from_u32(0xA0).unwrap(),
            // Unicode line separator (#x2028)
            b'L' => ret = char::from_u32(0x2028).unwrap(),
            // Unicode paragraph separator (#x2029)
            b'P' => ret = char::from_u32(0x2029).unwrap(),
            b'x' => code_length = 2,
            b'u' => code_length = 4,
            b'U' => code_length = 8,
            _ => {
                return Err(YamlError::new_str(
                    *start_mark,
                    "while parsing a quoted scalar, found unknown escape character",
                ));
            }
        }
        self.skip_n_non_blank(2);

        // Consume an arbitrary escape code.
        if code_length > 0 {
            // self.input.lookahead(code_length);
            let mut value = 0u32;
            for i in 0..code_length {
                let c = self.src.peek_nth(i);
                if !c.is_ascii_hexdigit() {
                    return Err(YamlError::new_str(
                        *start_mark,
                        "while parsing a quoted scalar, did not find expected hexadecimal number",
                    ));
                }
                value = (value << 4) + as_hex(c);
            }

            let Some(ch) = char::from_u32(value) else {
                return Err(YamlError::new_str(
                    *start_mark,
                    "while parsing a quoted scalar, found invalid Unicode character escape code",
                ));
            };
            ret = ch;

            self.skip_n_non_blank(code_length);
        }
        Ok(ret)
    }

    fn unroll_indent(&mut self, col: u32) {
        if self.flow_level > 0 {
            return;
        }

        while self.indent > col {
            // TODO: avoid unwrap
            let indent = self.indents.pop().unwrap();
            self.indent = indent.indent;
            if indent.needs_block_end {
                let span = Span::empty(self.mark);
                self.tokens.push_back(Token {
                    span,
                    token_type: BlockEnd,
                })
            }
        }
    }

    fn roll_indent(
        &mut self,
        col: u32,
        number: Option<usize>,
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
            let span = Span::empty(_mark);
            match number {
                Some(n) => self.insert_token(n - self.tokens_parsed, Token { span, token_type }),
                None => self.tokens.push_back(Token { span, token_type }),
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

    fn unroll_non_block_indents(&mut self) {
        while let Some(indent) = self.indents.last() {
            if indent.needs_block_end {
                break;
            }
            self.indent = indent.indent;
            self.indents.pop();
        }
    }

    fn insert_token(&mut self, pos: usize, token: Token<'input>) {
        let old_len = self.tokens.len();
        assert!(pos <= old_len);
        self.tokens.insert(pos, token);
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
            .ok_or_else(|| YamlError::new_str(self.mark, "recursion limit exceeded"))?;
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
            if sk.possible
                // If not in a flow construct, simple keys cannot span multiple lines.
                && self.flow_level == 0
                    && (sk.mark.line < self.mark.line || sk.mark.pos + 1024 < self.mark.pos)
            {
                if sk.required {
                    return Err(YamlError::new_str(self.mark, "simple key expect ':'"));
                }
                sk.possible = false;
            }
        }
        Ok(())
    }

    fn remove_simple_key(&mut self) -> ScanResult {
        let last = self.simple_keys.last_mut().unwrap();
        if last.possible && last.required {
            return Err(YamlError::new_str(self.mark, "simple key expected"));
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
                mark: self.mark,
                required,
                possible: true,
                token_number: self.tokens_parsed + self.tokens.len(),
            };

            self.simple_keys.pop();
            self.simple_keys.push(sk);
        }
    }

    fn scan_uri_escapes(&mut self, mark: &Marker) -> Result<Vec<u8>, YamlError> {
        let mut width = 0usize;
        let mut code = 0u32;
        loop {
            // self.src.lookahead(3);

            let c = self.src.peek_nth(1);
            let nc = self.src.peek_nth(2);

            if !(self.src.peek() == b'%' && c.is_ascii_hexdigit() && nc.is_ascii_hexdigit()) {
                return Err(YamlError::new_str(
                    *mark,
                    "while parsing a tag, found an invalid escape sequence",
                ));
            }

            let byte = (as_hex(c) << 4) + as_hex(nc);
            if width == 0 {
                width = match byte {
                    _ if byte & 0x80 == 0x00 => 1,
                    _ if byte & 0xE0 == 0xC0 => 2,
                    _ if byte & 0xF0 == 0xE0 => 3,
                    _ if byte & 0xF8 == 0xF0 => 4,
                    _ => {
                        return Err(YamlError::new_str(
                            *mark,
                            "while parsing a tag, found an incorrect leading UTF-8 byte",
                        ));
                    }
                };
                code = byte;
            } else {
                if byte & 0xc0 != 0x80 {
                    return Err(YamlError::new_str(
                        *mark,
                        "while parsing a tag, found an incorrect trailing UTF-8 byte",
                    ));
                }
                code = (code << 8) + byte;
            }

            self.skip_n_non_blank(3);

            width -= 1;
            if width == 0 {
                break;
            }
        }

        match char::from_u32(code) {
            Some(ch) => Ok(ch.to_string().as_bytes().to_vec()),
            None => Err(YamlError::new_str(
                *mark,
                "while parsing a tag, found an invalid UTF-8 codepoint",
            )),
        }
    }

    fn scan_directive_name(&mut self) -> Result<Vec<u8>, YamlError> {
        let start_mark = self.mark;
        let mut string = Vec::new();

        let n_chars = self.src.fetch_while_is_alpha(&mut string);
        self.mark.pos += n_chars;
        self.mark.col += n_chars as u32;

        if string.is_empty() {
            return Err(YamlError::new_str(
                start_mark,
                "while scanning a directive, could not find expected directive name",
            ));
        }

        if !is_blank_or_break(self.src.peek()) {
            return Err(YamlError::new_str(
                start_mark,
                "while scanning a directive, found unexpected non-alphabetical character",
            ));
        }

        Ok(string)
    }

    fn scan_version_directive_value(
        &mut self,
        marker: &Marker,
    ) -> Result<Token<'input>, YamlError> {
        let n_blanks = self.src.skip_while_blank();
        self.mark.pos += n_blanks;
        self.mark.col += n_blanks as u32;

        let major = self.scan_version_directive_number(marker)?;

        if self.src.peek() != b'.' {
            return Err(YamlError::new_str(
                *marker,
                "while scanning a YAML directive, did not find expected digit or '.' character",
            ));
        }
        self.skip_non_blank();

        let minor = self.scan_version_directive_number(marker)?;

        Ok(Token {
            span: Span::new(*marker, self.mark),
            token_type: TokenType::VersionDirective { major, minor },
        })
    }

    fn scan_tag_directive_value(&mut self, mark: &Marker) -> Result<Token<'input>, YamlError> {
        let n_blanks = self.src.skip_while_blank();
        self.mark.pos += n_blanks;
        self.mark.col += n_blanks as u32;

        let handle = String::from_utf8(self.scan_tag_handle(true, mark)?)
            .map_err(|_| YamlError::new_str(*mark, "Error decoding tag handle as UTF-8"))?
            .into();

        let n_blanks = self.src.skip_while_blank();
        self.mark.pos += n_blanks;
        self.mark.col += n_blanks as u32;

        let prefix = String::from_utf8(self.scan_tag_prefix(mark)?)
            .map_err(|_| YamlError::new_str(*mark, "Error decoding tag prefix as UTF-8"))?
            .into();

        // self.src.lookahead(1);

        if self.src.next_is_blank_or_break() {
            Ok(Token {
                span: Span::new(*mark, self.mark),
                // SAFETY: handle and prefix must not contain invalid UTF8
                token_type: TokenType::TagDirective { prefix, handle },
            })
        } else {
            Err(YamlError::new_str(
                *mark,
                "while scanning TAG, did not find expected whitespace or line break",
            ))
        }
    }

    fn scan_version_directive_number(&mut self, mark: &Marker) -> Result<u8, YamlError> {
        let mut val = 0;
        let mut length = 0usize;
        while self.src.peek().is_ascii_digit() {
            let digit = self.src.peek() - b'0';
            if length + 1 > 9 {
                return Err(YamlError::new_str(
                    *mark,
                    "while scanning a YAML directive, found extremely long version number",
                ));
            }
            length += 1;
            val = val * 10 + digit;
            self.skip_non_blank();
        }

        if length == 0 {
            return Err(YamlError::new_str(
                *mark,
                "while scanning a YAML directive, did not find expected version number",
            ));
        }

        Ok(val)
    }

    fn scan_tag_prefix(&mut self, start_mark: &Marker) -> Result<Vec<u8>, YamlError> {
        let mut string = Vec::new();

        if self.src.peek() == b'!' {
            // If we have a local tag, insert and skip `!`.
            string.push(self.src.peek());
            self.skip_non_blank();
        } else if !is_tag_char(self.src.peek()) {
            // Otherwise, check if the first global tag character is valid.
            return Err(YamlError::new_str(
                *start_mark,
                "invalid global tag character",
            ));
        } else if self.src.peek() == b'%' {
            // If it is valid and an escape sequence, escape it.
            string.extend(self.scan_uri_escapes(start_mark)?);
        } else {
            // Otherwise, push the first character.
            string.push(self.src.peek());
            self.skip_non_blank();
        }

        while is_uri_char(self.src.peek()) {
            if self.src.peek() == b'%' {
                string.extend(self.scan_uri_escapes(start_mark)?);
            } else {
                string.push(self.src.peek());
                self.skip_non_blank();
            }
        }

        Ok(string)
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
