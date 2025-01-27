use std::collections::VecDeque;
use std::ops::ControlFlow::{self, Break, Continue};

use ErrorType::NoDocStartAfterTag;
use SpanToken::{DocumentStart, Separator, Space};

use crate::tokenizer::reader::IndentType::{EndInstead, EqualIndent, LessOrEqualIndent};
use crate::tokenizer::reader::{is_flow_indicator, is_whitespace, Reader, StrReader};
use crate::tokenizer::scanner::ParserState::{
    BlockKey, BlockMap, BlockSeq, FlowKey, FlowMap, FlowSeq, PreDocStart, RootBlock,
};
use crate::tokenizer::scanner::SpanToken::{
    Directive, ErrorToken, KeyEnd, MappingEnd, MappingStart, MarkEnd, MarkStart, NewLine,
    SequenceEnd, SequenceStart,
};
use crate::tokenizer::ErrorType::{ExpectedIndent, UnexpectedSymbol};
use crate::tokenizer::{DirectiveType, ErrorType};

use super::reader::is_newline;

#[derive(Clone)]
pub struct Scanner {
    pub(crate) curr_state: ParserState,
    pub stream_end: bool,
    tokens: VecDeque<SpanToken>,
    stack: VecDeque<ParserState>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            stream_end: false,
            tokens: VecDeque::new(),
            curr_state: PreDocStart,
            stack: VecDeque::new(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum ParserState {
    PreDocStart,
    RootBlock,
    FlowSeq(usize),
    FlowMap(usize),
    FlowKey(usize, bool),
    BlockSeq(usize),
    BlockMap(usize),
    BlockKey(usize),
    AfterDocEnd,
}

impl ParserState {
    pub(crate) fn indent(&self) -> usize {
        match self {
            FlowKey(ind, _)
            | FlowMap(ind)
            | FlowSeq(ind)
            | BlockKey(ind)
            | BlockSeq(ind)
            | BlockMap(ind) => *ind,
            _ => 0,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match &self {
            FlowKey(_, _) | FlowSeq(_) | FlowMap(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_flow(&self) -> bool {
        match &self {
            FlowMap(_) | FlowKey(_, _) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_implicit(&self) -> bool {
        match &self {
            FlowKey(_, true) => true,
            _ => false,
        }
    }
}

impl Scanner {
    #[inline]
    pub fn peek_token(&self) -> Option<SpanToken> {
        match self.tokens.front() {
            Some(&x) => Some(x),
            None => None,
        }
    }

    #[inline(always)]
    pub fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn fetch_next_token<R: Reader>(&mut self, reader: &mut R) {
        self.scan_to_next_token(reader, true);
        match self.curr_state {
            PreDocStart => {
                if !reader.peek_byte_is(b'%') {
                    self.curr_state = RootBlock;
                    return;
                }

                if reader.try_read_slice_exact("%YAML") {
                    reader.skip_space_tab(true);
                    if let Some(x) = reader.find_next_whitespace() {
                        self.tokens.push_back(Directive(DirectiveType::Yaml));
                        self.tokens.push_back(MarkStart(reader.pos()));
                        self.tokens.push_back(MarkEnd(reader.pos() + x));

                        reader.consume_bytes(x);
                        reader.read_line();
                    }
                } else {
                    let tag = if reader.try_read_slice_exact("%TAG") {
                        Directive(DirectiveType::Tag)
                    } else {
                        Directive(DirectiveType::Reserved)
                    };
                    reader.skip_space_tab(true);
                    let x = reader.read_non_comment_line();
                    if x.0 != x.1 {
                        self.tokens.push_back(tag);
                        self.tokens.push_back(MarkStart(x.0));
                        self.tokens.push_back(MarkEnd(x.1));
                    }
                }
                if reader.try_read_slice_exact("---") {
                    self.tokens.push_back(DocumentStart)
                } else {
                    self.tokens.push_back(ErrorToken(NoDocStartAfterTag))
                }
            }
            RootBlock => match reader.peek_byte() {
                Some(b'{') => self.fetch_flow_col(reader, 0),
                Some(b'[') => self.fetch_flow_col(reader, 0),
                Some(b'&') => self.fetch_alias(reader),
                Some(b'*') => self.fetch_anchor(reader),
                Some(b':') => self.fetch_block_map(reader),
                Some(b'-') => self.switch_to_block_seq(reader, 0),
                Some(b'?') => self.fetch_block_map_key(reader),
                Some(b'!') => self.fetch_tag(reader),
                Some(b'>') => self.fetch_block_scalar(reader, false),
                Some(b'|') => self.fetch_block_scalar(reader, false),
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(x) => {
                    if x != b']' && x != b'}' && x != b'@' {
                        self.fetch_plain_scalar(reader, FlowSeq(0));
                    } else {
                        reader.consume_bytes(1);
                        self.tokens
                            .push_back(ErrorToken(UnexpectedSymbol(x as char)))
                    }
                }
                None => self.stream_end = true,
            },
            BlockSeq(indent) => match reader.peek_byte() {
                Some(b'-') => self.fetch_block_seq(reader, indent + 1),
                Some(_) => {
                    self.fetch_plain_scalar(reader, FlowSeq(indent + 1));
                }
                _ => todo!(),
            },
            FlowSeq(indent) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, indent + 1),
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(SequenceEnd);
                    self.pop_state();
                }
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken(UnexpectedSymbol('}')));
                }
                Some(b',') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(Separator);
                }
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b':') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingStart);
                    self.push_state(FlowKey(indent, true));
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader, self.curr_state);
                }
                None => self.stream_end = true,
            },
            FlowMap(indent) | FlowKey(indent, _) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, indent + 1),
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingEnd);
                    self.pop_state();
                }
                Some(b':') => self.process_map_key(reader, indent),
                Some(b']') => {
                    if self.is_prev_sequence() {
                        self.tokens.push_back(MappingEnd);
                        self.pop_state();
                    } else {
                        reader.consume_bytes(1);
                        self.tokens.push_back(ErrorToken(UnexpectedSymbol(']')));
                    }
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b',') => reader.consume_bytes(1),
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader, self.curr_state);
                }
                None => self.stream_end = true,
            },
            _ => {}
        }

        if reader.eof() {
            self.stream_end = true;
            return;
        }
    }

    fn scan_to_next_token<R: Reader>(&mut self, reader: &mut R, allow_tab: bool) {
        loop {
            reader.skip_space_tab(allow_tab);

            // read comment line
            if reader.peek_byte_is(b'#') {
                reader.read_line();
                continue;
            }

            // if not end of file read new line or space/tab in next loop
            if reader.eof() || reader.read_break().is_none() {
                break;
            }
        }
    }

    fn fetch_flow_col<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        let peek = reader.peek_byte().unwrap_or(b'\0');
        reader.consume_bytes(1);

        if reader.col() != 0 {
            reader.skip_space_tab(true);
        }

        if peek == b'[' {
            self.tokens.push_back(SequenceStart);
            self.push_state(FlowSeq(indent));
        } else if peek == b'{' {
            if reader.col() != 0 {
                reader.skip_space_tab(true);
            }
            if reader.peek_byte_is(b'?') {
                self.push_state(FlowKey(indent, false));
            } else {
                self.push_state(FlowKey(indent, true));
            }
            self.tokens.push_back(MappingStart);
        }
    }

    #[inline]
    fn push_state(&mut self, state: ParserState) {
        self.stack.push_back(self.curr_state);
        self.curr_state = state;
    }

    #[inline]
    fn pop_state(&mut self) {
        match self.stack.pop_back() {
            Some(x) => self.curr_state = x,
            None => self.curr_state = ParserState::AfterDocEnd,
        }
    }

    fn fetch_alias<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }

    fn fetch_anchor<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }

    fn fetch_block_map<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }

    fn switch_to_block_seq<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        if reader.peek_byte_at_check(1, is_whitespace) {
            let new_indent: usize = reader.col();
            if reader.peek_byte_at_check(1, is_newline) {
                reader.consume_bytes(1);
                reader.read_break();
            } else {
                reader.consume_bytes(2);
            }

            if new_indent > indent {
                self.push_state(BlockSeq(new_indent));
            }
        } else {
            self.fetch_plain_scalar(reader, FlowSeq(indent + 1));
        }
    }

    
    fn fetch_block_seq<R: Reader>(&self, reader: &mut R, indent: usize) {
        todo!()
    }

    fn fetch_block_map_key<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }

    fn fetch_tag<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }

    fn fetch_block_scalar<R: Reader>(&mut self, reader: &mut R, literal: bool) {
        todo!()
    }

    fn fetch_quoted_scalar<R: Reader>(&mut self, reader: &mut R, quote: u8) {
        let mut start = reader.pos();
        let mut first = 1;
        reader.consume_bytes(1);
        while let Some(offset) = reader.find_fast3_iter(quote, b'\r', b'\n') {
            match reader.peek_byte_at(offset) {
                Some(b'\r') | Some(b'\n') => {
                    if offset > 0 {
                        self.tokens.push_back(MarkStart(start));
                        self.tokens.push_back(MarkEnd(start + offset + first));
                        self.tokens.push_back(Space);
                    }
                    reader.read_line();
                    reader.skip_space_tab(self.curr_state.is_implicit());
                    start = reader.pos();
                    first = 0;
                }
                Some(_) => {
                    // consume offset and the next quote
                    reader.consume_bytes(offset + 1);
                    self.tokens.push_back(MarkStart(start));
                    self.tokens.push_back(MarkEnd(start + offset + first));
                    break;
                }
                None => {}
            };
        }
    }

    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R, context: ParserState) {
        let mut is_multiline = !context.is_implicit();
        let indent = context.indent();

        // assume first char will be correct and consume it
        self.read_plain_one_line(reader);

        // if multiline then we process next plain scalar
        while {
            is_multiline &= !reader.peek_byte_at_check(0, is_flow_indicator);
            is_multiline && !reader.eof()
        } {
            // separate in line
            if reader.col() != 0 {
                reader.skip_space_tab(true);
            }
            // b-l-folded
            let folded_str = self.try_read_folded(reader, &FlowMap(indent));
            if folded_str.is_empty() {
                break;
            }

            //s-flow-line-prefix
            if reader.try_read_indent(EqualIndent(indent)).is_equal() {
                if reader.col() != 0 {
                    reader.skip_space_tab(true);
                }
                if self.read_plain_one_line(reader) {
                    self.tokens.extend(folded_str);
                }
            } else {
                self.tokens.push_back(ErrorToken(ExpectedIndent(indent)));
            }
        }
    }

    fn try_read_folded<R: Reader>(
        &mut self,
        reader: &mut R,
        context: &ParserState,
    ) -> Vec<SpanToken> {
        let mut tokens = vec![];


        if reader.read_break().is_none() {
            tokens.push(ErrorToken(ErrorType::ExpectedNewlineInFolded));
        } else {
            let mut break_as_space = true;
            while reader.peek_byte_is( b' ')  {
                match reader.try_read_indent(LessOrEqualIndent(context.indent())) {
                    EndInstead => break,
                    EqualIndent(_) if context.is_flow() => {
                        // separate in line
                        if reader.col() != 0 {
                            reader.skip_space_tab(true);
                        }
                    }
                    _ => {}
                }

                break_as_space = false;

                if let Some(_) = reader.read_break() {
                    tokens.push(NewLine);
                }
            }

            if break_as_space {
                tokens.push(Space);
            }
        };

        tokens
    }

    fn read_plain_one_line<R: Reader>(&mut self, reader: &mut R) -> bool {
        let start = reader.pos();
        let in_flow_collection = self.curr_state.in_flow_collection();
        let mut offset = 0;
        while !reader.is_eof(offset as usize) {
            offset += reader.skip_space_tab(true);
            let read = reader.position_until(offset, |pos, x0, x1| {
                is_invalid_plain_scalar(pos, x0, x1, in_flow_collection)
            });
            if read != 0 {
                offset += read;
                if in_flow_collection
                    && reader.peek_byte_at(offset).map_or(false, is_flow_indicator)
                {
                    break;
                }
            } else {
                break;
            }
        }
        if offset > 0 {
            self.tokens.push_back(MarkStart(start));
            self.tokens.push_back(MarkEnd(start + offset));
            reader.consume_bytes(offset as usize);
        }
        return offset > 0;
    }

    fn fetch_explicit_map<R: Reader>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart);
        }

        if !reader.peek_byte_at_check(1, is_whitespace) {
            self.fetch_plain_scalar(reader, self.curr_state);
            return;
        }
        reader.consume_bytes(1);
        reader.skip_space_tab(true);
    }

    fn process_map_key<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);

        if self.is_key() {
            self.curr_state = FlowMap(indent);
            self.tokens.push_back(KeyEnd);
        } else {
            self.fetch_plain_scalar(reader, self.curr_state);
        }
    }

    #[inline]
    fn is_prev_sequence(&self) -> bool {
        match self.stack.back() {
            Some(FlowSeq(_)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_map(&self) -> bool {
        match self.curr_state {
            FlowMap(_) | FlowKey(_, _) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_key(&self) -> bool {
        match self.curr_state {
            FlowKey(_, _) => true,
            _ => false,
        }
    }
}

#[inline]
fn is_invalid_plain_scalar(
    pos: usize,
    x0: u8,
    x1: u8,
    in_flow_collection: bool,
) -> ControlFlow<usize, usize> {
    if in_flow_collection {
        if is_flow_indicator(x0) {
            return Break(pos);
        } else if is_flow_indicator(x1) && is_whitespace(x0) {
            return Break(pos);
        } else if is_flow_indicator(x1) {
            return Break(pos + 1);
        }
    }

    // ns-plain-char  prevent ` #`
    if is_whitespace(x0) && x1 == b'#' {
        return Break(pos);
    }

    // ns-plain-char prevent `: `
    // or `:{`  in flow collections
    if x0 == b':' && (is_whitespace(x1) || (in_flow_collection && is_flow_indicator(x1))) {
        return Break(pos);
    }

    if is_whitespace(x0) {
        return Break(pos);
    } else if is_whitespace(x1) {
        return Break(pos + 1);
    };

    Continue(pos + 1)
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    ErrorToken(ErrorType),
    MarkStart(usize),
    MarkEnd(usize),
    NewLine,
    Space,
    Directive(DirectiveType),
    Alias,
    Separator,
    KeyEnd,
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd,
    DocumentStart,
    DocumentEnd,
}
