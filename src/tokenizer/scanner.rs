use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};

use ErrorType::NoDocStartAfterTag;
use SpanToken::DocStart;
use State::*;

use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::is_empty;
use crate::tokenizer::iter::{ErrorType, StrIterator};
use crate::tokenizer::iter::ErrorType::UnexpectedSymbol;
use crate::tokenizer::reader::{IndentType, is_flow_indicator, is_tab_space, is_whitespace, Reader, StrReader};
use crate::tokenizer::scanner::QuoteType::{Double, Single};
use crate::tokenizer::scanner::SpanToken::{ErrorToken, Scalar};
use crate::tokenizer::scanner::State::{RootBlock, StreamStart};

#[derive(Clone)]
pub struct Scanner {
    pub(crate) curr_state: State,
    pub(crate) stream_end: bool,
    tokens: VecDeque<SpanToken>,
    indents: VecDeque<i32>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            indents: VecDeque::new(),
            stream_end: false,
            tokens: VecDeque::new(),
            curr_state: StreamStart,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum State {
    StreamStart,
    PreDocStart,
    RootBlock,
    BlockKey,
    FlowKey,
    FlowIn,
    OutsideFlow(u32),
    Block(u32),
    AfterDocEnd,
}

impl State {
    #[inline]
    pub fn is_flow_or_simple_key(&self) -> bool {
        match self {
            FlowKey | BlockKey | RootBlock => true,
            _ => false,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match self {
            FlowKey | FlowIn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn in_key(&self) -> bool {
        match self {
            FlowKey | BlockKey => true,
            _ => false,
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
        self.scan_to_next_token(reader);

        match self.curr_state {
            StreamStart => {
                self.tokens.push_back(SpanToken::StreamStart);
                self.curr_state = PreDocStart;
            }
            RootBlock => match reader.peek_byte() {
                Some(b'{') => {
                    self.fetch_flow_map(reader);
                }
                Some(b'[') => {
                    self.fetch_flow_seq(reader);
                }
                Some(b'&') => {
                    self.fetch_flow_alias(reader);
                }
                Some(b'*') => {
                    self.fetch_anchor(reader);
                }
                Some(b':') => {
                    self.fetch_block_map(reader);
                }
                Some(b'-') => {
                    self.fetch_block_seq(reader);
                }
                Some(b'?') => {
                    self.fetch_block_maq(reader);
                }
                Some(b'!') => {
                    self.fetch_tag(reader);
                }
                Some(b'>') => {
                    self.fetch_block_scalar(reader, false);
                }
                Some(b'|') => {
                    self.fetch_block_scalar(reader, false);
                }
                Some(b'\'') => {
                    self.fetch_flow_scalar(reader, Single);
                }
                Some(b'"') => {
                    self.fetch_flow_scalar(reader, Double);
                }
                Some(b'#') => {
                    reader.read_line();
                }
                Some(x) => {
                    if x != b']' && x != b'}' && x != b'@' {
                        self.curr_state = OutsideFlow(0);
                        self.fetch_plain_scalar(reader);
                    } else {
                        self.tokens.push_back(ErrorToken(UnexpectedSymbol))
                    }
                }
                _ => {}
            },
            PreDocStart => {
                if !reader.peek_byte_is(b'%') {
                    self.curr_state = RootBlock;
                    return;
                }

                if reader.try_read_slice_exact("%YAML") {
                    reader.skip_space_tab(true);
                    if let Some(x) = reader.find_next_whitespace() {
                        self.tokens.push_back(SpanToken::Directive(
                            DirectiveType::Yaml,
                            reader.pos(),
                            reader.pos() + x,
                        ));
                        reader.consume_bytes(x);
                        reader.read_line();
                    }
                } else {
                    let tag = if reader.try_read_slice_exact("%TAG") {
                        DirectiveType::Tag
                    } else {
                        DirectiveType::Reserved
                    };
                    reader.skip_space_tab(true);
                    let x = reader.read_non_comment_line();
                    if x.0 != x.1 {
                        self.tokens.push_back(SpanToken::Directive(tag, x.0, x.1));
                    }
                }
                if reader.try_read_slice_exact("---") {
                    self.tokens.push_back(DocStart)
                } else {
                    self.tokens.push_back(ErrorToken(NoDocStartAfterTag))
                }
            }
            _ => {}
        }

        if reader.eof() {
            self.tokens.push_back(SpanToken::StreamEnd);
            self.stream_end = true;
            return;
        }
    }

    fn scan_to_next_token<R: Reader>(&mut self, reader: &mut R) {
        loop {
            reader.skip_space_tab(self.curr_state.is_flow_or_simple_key());

            // read comment line
            if reader.peek_byte_is(b'#') {
                reader.read_line();
                break;
            }

            // if not end of file read new line or space/tab in next loop
            if reader.eof() || reader.read_break().is_none() {
                break;
            }
        }
    }
    fn fetch_flow_map<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }
    fn fetch_flow_seq<R: Reader>(&mut self, reader: &mut R) {
        todo!()
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
    fn fetch_block_seq<R: Reader>(&mut self, reader: &mut R) {
        todo!()
    }
    fn fetch_block_maq<R: Reader>(&mut self, reader: &mut R) {
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
    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R) {
        let mut is_multiline = self.curr_state.in_key();
        let start = reader.pos();

        let in_flow_collection = self.curr_state.in_flow_collection();

        // assume first char will be correct and consume it
        let mut end = reader.pos();
        self.read_plain_one_line(reader, in_flow_collection, &mut end);

        // if multiline then we process next plain scalar
        while is_multiline {
            // separate in line
            if reader.col() != 0 {
                reader.skip_space_tab(true);
            }
            // b-l-folded
            self.read_folded(reader, self.get_indent(), true);
        }

        self.tokens.push_back(Scalar(start, end));
    }



    #[inline]
    fn read_folded<R: Reader>(&mut self, reader: &mut R, indent: u32, in_flow: bool) {
        // try read break
        if is_empty(reader.read_line()) {
            self.tokens.push_back(ErrorToken(ErrorType::ExpectedNewlineInFolded));
            return;
        }
       // l-empty

        while reader.try_read_indent(indent, IndentType::LessOrEqual) {

        }

    }

    #[inline]
    fn read_plain_one_line<R: Reader>(&mut self, reader: &mut R, in_flow_collection: bool, end: &mut usize) {
        while !reader.eof() {
            *end += reader.skip_space_tab(true);
            let read = reader.position_until(|pos, x0, x1| {
                if is_whitespace(x0) {
                    return Break(pos);
                } else if is_whitespace(x1) {
                    return Break(pos + 1);
                };

                // ns-plain-char  prevent ` #`
                if is_whitespace(x0) && x1 == b'#' {
                    return Break(pos);
                }

                // ns-plain-char prevent `: `
                // or `:{`  in flow collections
                if x0 == b':'
                    && (is_whitespace(x1) || (in_flow_collection && is_flow_indicator(x1)))
                {
                    self.curr_state = if in_flow_collection {
                        FlowKey
                    } else {
                        BlockKey
                    };
                    self.tokens.push_back(SpanToken::MapKey);
                    return Break(pos);
                }

                Continue(pos + 1)
            });
            reader.read_line();
            if read == 0 {
                return;
            } else {
                *end += read;
            }
        }
    }

    #[inline]
    fn get_indent(&self) -> u32 {
        match self.curr_state {
            OutsideFlow(x) => x,
            _ => 0,
        }
    }

    #[inline]
    fn incr_block(&mut self) {
        match self.curr_state {
            RootBlock => self.curr_state = Block(0),
            Block(x) => self.curr_state = Block(x + 1),
            _ => {}
        }
    }
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    MapKey,
    ErrorToken(ErrorType),
    Scalar(usize, usize),
    Directive(DirectiveType, usize, usize),
    DocStart,
    DocEnd,
    StreamStart,
    StreamEnd,
}
