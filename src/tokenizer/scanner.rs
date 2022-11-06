use std::collections::VecDeque;

use State::{InBlockMap, InBlockScalar, InBlockSeq, InFlowMap, InFlowSeq};

use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::iter::{ErrorType, StrIterator};
use crate::tokenizer::reader::{is_indicator, is_whitespace, Reader, StrReader};
use crate::tokenizer::scanner::QuoteType::{Double, Plain, Single};
use crate::tokenizer::scanner::ScannerContext::{BlockIn, FlowIn, FlowKey, FlowOut};
use crate::tokenizer::scanner::SpanToken::{ErrorToken, Scalar};
use crate::tokenizer::scanner::State::{BlockNode, Failure, InFlowScalar, StreamEnd, StreamStart};

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

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum ScannerContext {
    BlockIn,
    BlockOut,
    BlockKey,
    FlowIn,
    FlowOut,
    FlowKey,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum State {
    StreamStart,
    DocStart,
    BlockNode,
    StreamEnd,
    InFlowSeq,
    InBlockSeq,
    InFlowMap,
    InBlockMap,
    InMap,
    InFlowScalar(QuoteType),
    InBlockScalar,
    Failure,
}

impl State {
    pub fn is_flow_or_simple_key(&self) -> bool {
        match self {
            InFlowScalar(_) | InFlowMap | InFlowSeq => true,
            BlockNode => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum QuoteType {
    Plain,
    Single,
    Double,
}

impl Default for QuoteType {
    fn default() -> Self {
        Plain
    }
}

// #[derive(PartialEq)]
// pub enum Control {
//     Continue,
//     Eof,
//     End,
// }

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

    pub(crate) fn fetch_next_token<R: Reader>(&mut self, reader: &mut R)  {
        if self.curr_state == StreamStart {
            self.tokens.push_back(SpanToken::StreamStart);
            self.curr_state = BlockNode;
        }

        self.scan_to_next_token(reader);

        if reader.eof() {
            self.tokens.push_back(SpanToken::StreamEnd);
            self.stream_end = true;
            return;
        }

        if reader.peek_byte_is(b'%') {
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
        }

        if reader.try_read_slice_exact("---") {
            self.tokens.push_back(SpanToken::DocStart);
            self.curr_state = BlockNode;
        }

        if reader.try_read_slice_exact("...") {
            self.tokens.push_back(SpanToken::DocEnd);
            self.curr_state = State::DocStart;
        }
    }

    fn scan_to_next_token<R: Reader>(&mut self, reader: &mut R) {
        let mut cont = true;
        while cont {
            reader.skip_space_tab(self.curr_state.is_flow_or_simple_key());

            if reader.peek_byte_is(b'#') {
                reader.read_line();
            } else {
                cont = false;
            }
        };
    }
}


#[derive(Copy, Clone)]
pub enum SpanToken {
    ErrorToken(ErrorType),
    Scalar(usize, usize),
    Directive(DirectiveType, usize, usize),
    DocStart,
    DocEnd,
    StreamStart,
    StreamEnd,
}
