use crate::tokenizer::char_utils::{is_blank, is_blank_or_break, is_break, is_flow};
use TokenType::FlowSequenceEnd;
use alloc::borrow::{Cow, ToOwned};
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use yam_common::ScalarType::Plain;
use yam_common::TokenType::{
    BlockEnd, FlowMappingEnd, FlowMappingStart, FlowSequenceStart, StreamEnd,
};
use yam_common::{Marker, ScanResult, TokenType, YamlError, YamlResult};

pub trait Source {
    #[must_use]
    fn peek(&self) -> u8;

    #[must_use]
    fn peek_char(&self) -> char;

    #[must_use]
    fn peek_nth(&self, n: usize) -> u8;

    fn skip(&mut self, n: usize);

    #[must_use]
    fn bufmaxlen(&self) -> usize;

    fn skip_ws_to_eol(&mut self, skip_tabs: SkipTabs) -> (u32, Result<SkipTabs, &'static str>);
    fn next_byte_is(&self, chr: u8) -> bool {
        self.peek() == chr
    }

    fn nth_byte_is(&self, n: usize, chr: u8) -> bool {
        self.peek_nth(n) == chr
    }

    fn peek_two(&self) -> [u8; 2] {
        [self.peek(), self.peek_nth(1)]
    }

    fn next_is_three(&self, chr: u8) -> bool {
        self.peek() == chr && self.peek_nth(1) == chr && self.peek_nth(2) == chr
    }

    #[must_use]
    fn next_is_flow(&self) -> bool {
        is_flow(self.peek())
    }

    #[must_use]
    fn next_is_break(&self) -> bool {
        is_break(self.peek())
    }

    #[must_use]
    fn next_is_blank(&self) -> bool {
        is_blank(self.peek())
    }

    fn skip_while_non_breakz(&mut self) -> usize {
        let mut count = 0;
        while !is_break(self.peek()) {
            count += 1;
            self.skip(1);
        }
        count
    }

    fn next_is_blank_or_break(&self) -> bool {
        is_blank_or_break(self.peek())
    }

    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peek_nth(1);
        match self.peek() {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            b':' if is_blank_or_break(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    fn next_is_document_indicator(&self) -> bool {
        (self.next_is_three(b'-') || self.next_is_three(b'.'))
            && is_blank_or_break(self.peek_nth(3))
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SkipTabs {
    Yes,
    No,
    Result {
        any_tabs: bool,
        any_whitespace: bool,
    },
}

impl SkipTabs {
    pub(crate) fn found_tabs(&self) -> bool {
        matches!(self, SkipTabs::Result { any_tabs: true, .. })
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Span {
    pub start: Marker,
    pub end: Marker,
}

impl Span {}

impl Span {
    fn new(start: Marker, end: Marker) -> Self {
        Span { start, end }
    }

    fn empty(mark: Marker) -> Self {
        Span {
            start: mark,
            end: mark,
        }
    }
}

pub struct Token<'input> {
    span: Span,
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
    stream_end_produced: bool,

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
        self.src.next_is_three(b'-') && is_blank_or_break(self.src.peek_nth(4))
    }

    fn next_is_document_end(&mut self) -> bool {
        self.src.next_is_three(b'.') && is_blank_or_break(self.src.peek_nth(4))
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

        let span = self.get_span(start_mark);
        self.tokens.push_back(Token { span, token_type });

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
            return Err(YamlError::scanner_err(
                self.mark,
                r#""-" is only valid inside a block"#,
            ));
        }
        // Check if we are allowed to start a new entry.
        if !self.simple_key_allowed {
            return Err(YamlError::scanner_err(
                self.mark,
                "block sequence entries are not allowed in this context",
            ));
        }

        // ???, fixes test G9HC.
        if let Some(Token {
            span,
            token_type: TokenType::Anchor(..) | TokenType::TagDirective { .. },
        }) = self.tokens.back()
            && self.mark.col == 0
            && span.start.col == 0
            && self.indent > 0
        {
            return Err(YamlError::scanner_err(
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
            return Err(YamlError::scanner_err(
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
                return Err(YamlError::scanner_err(
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
            return Err(YamlError::scanner_err(
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
            Err(YamlError::scanner_err(self.mark, "expected whitespace"))
        } else {
            Ok(())
        }
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
        if !self.src.next_is_break() {
            Err(YamlError::scanner_err(
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
        result.map_err(|message| YamlError::scanner_err(self.mark, message))
    }

    #[inline]
    fn skip_linebreak(&mut self) {
        match self.src.peek_two() {
            [b'\r', b'\n'] => {
                self.mark.pos += 2;
                self.mark.col = 0;
                self.mark.line += 1;
                self.src.skip(2);
            }
            [b'\n', _] => {
                self.mark.pos += 1;
                self.mark.col = 0;
                self.mark.line += 1;
                self.src.skip(1);
            }
            _ => {}
        }
    }

    fn skip_blank(&self) {
        todo!()
    }

    fn skip_non_blank(&self) {
        todo!()
    }

    fn scan_directive(&mut self) -> YamlResult<Token<'input>> {
        todo!()
    }

    #[allow(clippy::too_many_lines)]
    fn scan_plain_scalar(&mut self) -> Result<Token<'input>, YamlError> {
        self.unroll_non_block_indents();
        let indent = self.indent + 1;
        let start_mark = self.mark;

        if self.flow_level > 0 && start_mark.col < indent {
            return Err(YamlError::scanner_err(
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
            if (self.leading_whitespace && self.src.next_is_document_indicator())
                || self.src.peek() == b'#'
            {
                break;
            }

            if self.flow_level > 0 && self.src.peek() == b'-' && is_flow(self.src.peek_nth(1)) {
                return Err(YamlError::scanner_err(
                    self.mark,
                    "plain scalar cannot start with '-' followed by ,[]{}",
                ));
            }

            if !self.src.next_is_blank_or_break()
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
                        if self.src.next_is_blank_or_break()
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
                        if !self.src.next_is_break() {
                            return Err(YamlError::scanner_err(
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
                        // TODO check this works
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
            Err(YamlError::scanner_err(
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

    fn unroll_indent(&mut self, col: u32) {
        if self.flow_level > 0 {
            return;
        }

        while self.indent >= col {
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
            let span = Span::empty(_mark);
            match number {
                Some(n) => {
                    self.insert_token(n as usize - self.tokens_parsed, Token { span, token_type })
                }
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
