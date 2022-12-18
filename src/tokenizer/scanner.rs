use std::collections::VecDeque;
use std::ops::ControlFlow::{self, Break, Continue};

use ErrorType::NoDocStartAfterTag;
use SpanToken::DocumentStart;

use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::iter::ErrorType::{ExpectedIndent, UnexpectedSymbol};
use crate::tokenizer::iter::{ErrorType, StrIterator};
use crate::tokenizer::reader::IndentType::{EqualIndent, LessOrEqualIndent};
use crate::tokenizer::reader::{is_flow_indicator, is_whitespace, Reader, StrReader};
use crate::tokenizer::scanner::NodeContext::{BlockIn, BlockKey, FlowIn, FlowKey, FlowOut};
use crate::tokenizer::scanner::ParserState::{FlowMap, FlowSeq, PreDocStart, RootBlock};
use crate::tokenizer::scanner::QuoteType::{Double, Single};
use crate::tokenizer::scanner::SpanToken::{Directive, ErrorToken, MappingEnd, MappingStart, MarkEnd, MarkStart, SequenceEnd, SequenceStart};

#[derive(Clone)]
pub struct Scanner {
    pub(crate) curr_state: ParserState,
    pub(crate) stream_end: bool,
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
    FlowSeq(NodeContext, u32),
    FlowMap(NodeContext, u32),
    AfterDocEnd,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum NodeContext {
    FlowIn,
    FlowOut,
    FlowKey,
    BlockIn,
    BlockOut,
    BlockKey,
}

impl NodeContext {
    #[inline]
    pub fn in_implicit_key(&self) -> bool {
        match self {
            FlowKey | BlockKey => true,
            _ => false,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match *self {
            FlowKey | FlowIn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_flow(&self) -> bool {
        match *self {
            FlowOut | FlowIn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn to_flow(&self) -> NodeContext {
        match self {
            FlowOut | FlowIn => FlowIn,
            FlowKey | BlockKey => FlowKey,
            _ => panic!("Impossible state"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum QuoteType {
    Single,
    Double,
}

impl Default for QuoteType {
    fn default() -> Self {
        Single
    }
}

impl Scanner {
    pub fn from_str_reader(slice: &str) -> StrIterator<'_> {
        StrIterator {
            state: Default::default(),
            reader: StrReader::new(slice),
        }
    }

    #[inline(always)]
    pub(crate) fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub(crate) fn fetch_next_token<R: Reader>(&mut self, reader: &mut R) {
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
                Some(b'{') => self.fetch_flow_col(reader, FlowOut, 0),
                Some(b'[') => self.fetch_flow_col(reader, FlowOut, 0),
                Some(b'&') => self.fetch_flow_alias(reader),
                Some(b'*') => self.fetch_anchor(reader),
                Some(b':') => self.fetch_block_map(reader),
                Some(b'-') => self.fetch_block_seq(reader, BlockIn, 0, true),
                Some(b'?') => self.fetch_block_map_key(reader),
                Some(b'!') => self.fetch_tag(reader),
                Some(b'>') => self.fetch_block_scalar(reader, false),
                Some(b'|') => self.fetch_block_scalar(reader, false),
                Some(b'\'') => self.fetch_flow_scalar(reader, Single),
                Some(b'"') => self.fetch_flow_scalar(reader, Double),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(x) => {
                    if x != b']' && x != b'}' && x != b'@' {
                        self.fetch_plain_scalar(reader, BlockIn, 0);
                    } else {
                        reader.consume_bytes(1);
                        self.tokens
                            .push_back(ErrorToken(UnexpectedSymbol(x as char)))
                    }
                }
                None => self.stream_end = true,
            },
            FlowSeq(context, indent) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, context, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, context, indent + 1),
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(SequenceEnd);
                    self.pop_state();
                }
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken(UnexpectedSymbol('}')));
                }
                Some(b',') => reader.consume_bytes(1),
                Some(b'\'') => self.fetch_flow_scalar(reader, Single),
                Some(b'"') => self.fetch_flow_scalar(reader, Double),
                Some(_) => {
                    self.fetch_plain_scalar(reader, context, indent);
                }
                None => self.stream_end = true,
            },
            FlowMap(context, indent) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, context, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, context, indent + 1),
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingEnd);
                    self.pop_state();
                }
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken(UnexpectedSymbol(']')));
                }
                Some(b',') => reader.consume_bytes(1),
                Some(b'\'') => self.fetch_flow_scalar(reader, Single),
                Some(b'"') => self.fetch_flow_scalar(reader, Double),
                Some(_) => {
                    self.fetch_plain_scalar(reader, context, indent);
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

    fn fetch_flow_col<R: Reader>(&mut self, reader: &mut R, context: NodeContext, indent: u32) {
        let peek = reader.peek_byte().unwrap_or(b'\0');
        reader.consume_bytes(1);
        self.skip_separation_spaces(reader, context, indent);

        if peek == b'[' {
            self.tokens.push_back(SequenceStart);
            self.push_state(FlowSeq(context.to_flow(), indent));
        } else if peek == b'{' {
            self.tokens.push_back(MappingStart);
            self.push_state(FlowMap(context.to_flow(), indent));
        }
    }

    fn push_state(&mut self, state: ParserState) {
        self.stack.push_back(self.curr_state);
        self.curr_state = state;
    }

    fn pop_state(&mut self) {
        match self.stack.pop_front() {
            Some(x) => self.curr_state = x,
            None => self.curr_state = ParserState::AfterDocEnd,
        }
    }

    fn skip_separation_spaces<R: Reader>(
        &mut self,
        reader: &mut R,
        context: NodeContext,
        indent: u32,
    ) {
        let not_in_key = !matches!(context, FlowKey | BlockKey);
        if not_in_key {
            if reader.col() != 0 {
                if reader.peek_byte_is(b'#') {
                    reader.read_line();
                }
            }
            if !reader.try_read_indent(EqualIndent(indent)).is_ok() {
                self.tokens.push_back(ErrorToken(ExpectedIndent(indent)));
            }
        }
        if reader.col() != 0 {
            reader.skip_space_tab(true);
        }
    }

    fn fetch_flow_alias<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }
    fn fetch_anchor<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }
    fn fetch_block_map<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }
    fn fetch_block_seq<R: Reader>(
        &mut self,
        reader: &mut R,
        context: NodeContext,
        indent: i32,
        root: bool,
    ) {
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
    fn fetch_flow_scalar<R: Reader>(&mut self, reader: &mut R, quote: QuoteType) {
        todo!()
    }
    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R, context: NodeContext, indent: u32) {
        let is_multiline = !context.in_implicit_key();

        // assume first char will be correct and consume it
        self.read_plain_one_line(context, reader);

        // if multiline then we process next plain scalar
        if !reader.peek_byte_at_check(0, is_flow_indicator) {
            while is_multiline && !reader.eof() {
                // separate in line
                if reader.col() != 0 {
                    reader.skip_space_tab(true);
                }
                // b-l-folded
                self.read_folded(FlowIn, indent, reader);
    
                //s-flow-line-prefix
                if reader.try_read_indent(EqualIndent(indent)).is_ok() {
                    if reader.col() != 0 {
                        reader.skip_space_tab(true);
                    }
                    self.read_plain_one_line(context, reader);
                } else {
                    self.tokens.push_back(ErrorToken(ExpectedIndent(indent)));
                }
            }
        }
        
    }

    #[inline]
    fn read_folded<R: Reader>(&mut self, context: NodeContext, indent: u32, reader: &mut R) {
        // try read break
        if reader.read_break().is_none() {
            self.tokens
                .push_back(ErrorToken(ErrorType::ExpectedNewlineInFolded));
            return;
        }
        // l-empty
        while let Ok(x) = reader.try_read_indent(LessOrEqualIndent(indent)) {
            // must be block/line prefix
            match x {
                EqualIndent(_) if context.is_flow() => {
                    // separate in line
                    if reader.col() != 0 {
                        reader.skip_space_tab(true);
                    }
                }
                _ => {}
            }
            // b-as-line-feed expected
            if let Some(x) = reader.read_break() {
                self.tokens.push_back(MarkStart(x.0));
                self.tokens.push_back(MarkEnd(x.1));
                continue;
            } else if indent > 0 {
                self.tokens
                    .push_back(ErrorToken(ErrorType::ExpectedNewlineInFolded));
            }
            return;
        }
    }

    #[inline]
    fn read_plain_one_line<R: Reader>(&mut self, context: NodeContext, reader: &mut R) {
        let start = reader.pos();
        let in_flow_collection = context.in_flow_collection();
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
    }
}

#[inline]
fn is_invalid_plain_scalar(
    pos: usize,
    x0: u8,
    x1: u8,
    in_flow_collection: bool,
) -> ControlFlow<usize, usize> {
    if is_whitespace(x0) {
        return Break(pos);
    } else if is_whitespace(x1) {
        return Break(pos + 1);
    };

    if in_flow_collection {
        if is_flow_indicator(x0) {
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

    Continue(pos + 1)
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    ErrorToken(ErrorType),
    MarkStart(usize),
    MarkEnd(usize),
    Directive(DirectiveType),
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd,
    DocumentStart,
    DocumentEnd,
}
