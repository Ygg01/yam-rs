#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::hint::unreachable_unchecked;
use LexerState::PreDocStart;

use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::spanner::LexerState::{
    AfterDocEnd, BlockMap, BlockMapKeyExp, BlockMapVal, BlockMapValExp, BlockSeq, DirectiveSection,
    FlowKey, FlowKeyExp, FlowMap, FlowSeq, RootBlock,
};
use crate::tokenizer::spanner::LexerToken::*;
use crate::tokenizer::ErrorType;
use crate::tokenizer::ErrorType::UnexpectedSymbol;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{is_flow_indicator, is_newline};
use ErrorType::ExpectedIndent;

#[derive(Clone, Default)]
pub struct Lexer {
    pub(crate) curr_state: LexerState,
    pub stream_end: bool,
    pub directive: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    stack: Vec<LexerState>,
}

impl Lexer {
    pub(crate) fn extract(self) -> (VecDeque<usize>, Vec<ErrorType>) {
        (self.tokens, self.errors)
    }
}

pub trait StateSpanner<T> {}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum LexerState {
    #[default]
    PreDocStart,
    DirectiveSection,
    RootBlock,
    FlowSeq(u32),
    FlowMap(u32),
    FlowKey(u32),
    FlowKeyExp(u32),
    BlockSeq(u32),
    BlockMap(u32),
    BlockMapVal(u32),
    BlockMapKeyExp(u32),
    BlockMapValExp(u32),
    AfterDocEnd,
}

impl LexerState {
    #[inline]
    pub(crate) fn indent(&self, default: usize) -> u32 {
        match self {
            FlowKey(ind) | FlowKeyExp(ind) | FlowMap(ind) | FlowSeq(ind) | BlockSeq(ind)
            | BlockMap(ind) | BlockMapVal(ind) | BlockMapKeyExp(ind) | BlockMapValExp(ind) => *ind,
            RootBlock => default as u32,
            PreDocStart | AfterDocEnd | DirectiveSection => 0,
        }
    }

    #[inline]
    pub(crate) fn get_block_indent(&self, default: usize) -> usize {
        match self {
            BlockMapKeyExp(ind) | BlockMapValExp(ind) => *ind as usize,
            _ => default,
        }
    }

    #[inline]
    pub(crate) fn wrong_exp_indent(&self, curr_indent: usize) -> bool {
        match self {
            BlockMapKeyExp(ind) | BlockMapValExp(ind) => *ind as usize != curr_indent,
            _ => false,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match &self {
            FlowKey(_) | FlowKeyExp(_) | FlowSeq(_) | FlowMap(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_implicit(&self) -> bool {
        match &self {
            FlowKeyExp(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_new_block_col(&self, curr_indent: usize) -> bool {
        match &self {
            FlowKey(_) | FlowKeyExp(_) | FlowMap(_) | FlowSeq(_) => false,
            BlockMap(x) | BlockMapVal(x) | BlockMapKeyExp(x) | BlockMapVal(x)
                if *x as usize == curr_indent =>
            {
                false
            }
            _ => true,
        }
    }
}

impl Lexer {
    #[inline(always)]
    pub fn pop_token(&mut self) -> Option<usize> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub fn tokens(self) -> VecDeque<usize> {
        self.tokens
    }

    #[inline(always)]
    pub fn peek_token(&mut self) -> Option<usize> {
        self.tokens.front().copied()
    }

    #[inline(always)]
    pub fn peek_token_next(&mut self) -> Option<usize> {
        self.tokens.get(1).copied()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn fetch_next_token<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.skip_separation_spaces(true);
        match self.curr_state {
            PreDocStart => {
                if reader.peek_byte_is(b'%') {
                    self.curr_state = DirectiveSection;
                    return;
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                } else if reader.try_read_slice_exact("---") {
                    self.directive = true;
                    self.tokens.push_back(DocumentStart as usize);
                } else {
                    self.curr_state = RootBlock;
                }
                return;
            }
            DirectiveSection => {
                if !reader.try_read_yaml_directive(&mut self.tokens) {
                    if reader.try_read_slice_exact("---") {
                        self.tokens.push_back(DocumentStart as usize);
                        self.curr_state = RootBlock;
                        self.directive = true;
                        return;
                    } else if reader.peek_byte_is(b'#') {
                        reader.read_line();
                    }
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                }
            }
            RootBlock | BlockMap(_) | BlockMapVal(_) | BlockMapKeyExp(_) | BlockMapValExp(_)
            | BlockSeq(_) => {
                let indent = self.curr_state.indent(reader.col());
                let init_indent = match self.curr_state {
                    BlockMapKeyExp(ind) | BlockMapValExp(ind) => ind,
                    BlockMap(_) | BlockMapVal(_) => reader.col() as u32,
                    _ => indent,
                };
                match reader.peek_byte() {
                    Some(b'{') => self.fetch_flow_col(reader, indent as usize),
                    Some(b'[') => self.fetch_flow_col(reader, indent as usize),
                    Some(b'&') => reader.consume_anchor_alias(&mut self.tokens, AnchorToken),
                    Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                    Some(b':') if indent == 0 && reader.col() == 0 => {
                        reader.consume_bytes(1);
                        if self.curr_state == RootBlock {
                            self.tokens.push_back(MappingStart as usize);
                        }
                        if self.curr_state == BlockMap(0) || self.curr_state == RootBlock {
                            // Emit empty key if it's `:` on first colon of first element.
                            self.tokens.push_back(ScalarPlain as usize);
                            self.tokens.push_back(ScalarEnd as usize);
                        }
                        self.curr_state = BlockMapVal(0);
                    }
                    Some(b':') => {
                        reader.consume_bytes(1);
                        if let BlockMapKeyExp(x1) = self.curr_state {
                            self.curr_state = BlockMapValExp(x1);
                        } else if let BlockMap(x2) = self.curr_state {
                            self.curr_state = BlockMapVal(x2);
                        }
                    }
                    Some(b'-') => self.fetch_block_seq(reader, indent as usize),
                    Some(b'?') => self.fetch_block_map_key(reader, indent as usize),
                    Some(b'!') => self.fetch_tag(reader),
                    Some(b'|') => reader.read_block_scalar(
                        true,
                        &self.curr_state,
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'>') => reader.read_block_scalar(
                        false,
                        &self.curr_state,
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'\'') => reader.read_single_quote(false, &mut self.tokens),
                    Some(b'"') => reader.read_double_quote(false, &mut self.tokens),
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(x) => {
                        if x != b']' && x != b'}' && x != b'@' {
                            self.fetch_plain_scalar(reader, indent as usize, init_indent as usize);
                        } else {
                            reader.consume_bytes(1);
                            self.tokens.push_back(ErrorToken as usize);
                            self.errors.push(UnexpectedSymbol(x as char))
                        }
                    }
                    None => self.stream_end = true,
                }
            }
            FlowSeq(indent) => match reader.peek_byte() {
                Some(b'&') => reader.consume_anchor_alias(&mut self.tokens, AnchorToken),
                Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                Some(b'[') => self.fetch_flow_col(reader, (indent + 1) as usize),
                Some(b'{') => self.fetch_flow_col(reader, (indent + 1) as usize),
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(SequenceEnd as usize);
                    self.pop_state();
                }
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken as usize);
                    self.errors.push(UnexpectedSymbol('}'));
                }
                Some(b',') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ScalarEnd as usize);
                }
                Some(b'\'') => {
                    reader.read_single_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b'"') => {
                    reader.read_double_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b':') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingStart as usize);
                    self.push_state(FlowKeyExp(indent));
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader, indent as usize, reader.col());
                }
                None => self.stream_end = true,
            },
            FlowMap(indent) | FlowKey(indent) | FlowKeyExp(indent) => match reader.peek_byte() {
                Some(b'&') => reader.consume_anchor_alias(&mut self.tokens, AnchorToken),
                Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                Some(b'[') => self.fetch_flow_col(reader, (indent + 1) as usize),
                Some(b'{') => self.fetch_flow_col(reader, (indent + 1) as usize),
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingEnd as usize);
                    self.pop_state();
                }
                Some(b':') => self.process_map_key(reader, indent as usize),
                Some(b']') => {
                    if self.is_prev_sequence() {
                        self.tokens.push_back(MappingEnd as usize);
                        self.pop_state();
                    } else {
                        reader.consume_bytes(1);
                        self.tokens.push_back(ErrorToken as usize);
                        self.errors.push(UnexpectedSymbol(']'));
                    }
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b',') => {
                    reader.consume_bytes(1);
                }
                Some(b'\'') => {
                    reader.read_single_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b'"') => {
                    reader.read_double_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader, indent as usize, reader.col());
                }
                None => self.stream_end = true,
            },
            _ => {}
        }

        if reader.eof() {
            self.stream_end = true;
            self.stack.push(self.curr_state);
            for state in self.stack.iter().rev() {
                let x = match *state {
                    BlockSeq(_) => SequenceEnd,
                    BlockMap(_) | BlockMapKeyExp(_) => MappingEnd,
                    BlockMapVal(_) => {
                        // Empty element in block map
                        self.tokens.push_back(ScalarPlain as usize);
                        MappingEnd
                    }
                    DirectiveSection => {
                        self.errors.push(ErrorType::DirectiveEndMark);
                        ErrorToken
                    }
                    _ => continue,
                };
                self.tokens.push_back(x as usize);
            }
            if self.directive {
                self.tokens.push_back(DocumentEnd as usize);
            }
        }
    }

    fn fetch_flow_col<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        pub use LexerToken::*;

        let peek = reader.peek_byte().unwrap_or(b'\0');
        reader.consume_bytes(1);

        if reader.col() != 0 {
            reader.skip_space_tab(true);
        }

        if peek == b'[' {
            self.tokens.push_back(SequenceStart as usize);
            self.push_state(FlowSeq(indent as u32));
        } else if peek == b'{' {
            if reader.col() != 0 {
                reader.skip_space_tab(true);
            }
            if reader.peek_byte_is(b'?') {
                self.push_state(FlowKey(indent as u32));
            } else {
                self.push_state(FlowKeyExp(indent as u32));
            }
            self.tokens.push_back(MappingStart as usize);
        }
    }

    #[inline]
    fn push_state(&mut self, state: LexerState) {
        self.stack.push(self.curr_state);
        self.curr_state = state;
    }

    #[inline]
    fn pop_state(&mut self) {
        match self.stack.pop() {
            Some(x) => self.curr_state = x,
            None => self.curr_state = AfterDocEnd,
        }
    }

    fn fetch_block_seq<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        if let Some(new_state) = reader.read_block_seq(indent) {
            self.tokens.push_back(SequenceStart as usize);
            self.push_state(new_state);
        } else {
            self.fetch_plain_scalar(reader, indent, indent);
        }
    }

    fn fetch_block_map_key<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);
        self.push_state(BlockMapKeyExp(indent as u32));
        self.tokens.push_back(MappingStart as usize);
    }

    fn fetch_tag<B, R: Reader<B>>(&mut self, reader: &mut R) {
        pub use LexerToken::*;

        let start = reader.consume_bytes(1);
        if let Some((mid, end)) = reader.read_tag() {
            self.tokens.push_back(TagStart as usize);
            self.tokens.push_back(start);
            self.tokens.push_back(mid);
            self.tokens.push_back(end);
            reader.consume_bytes(end - start);
        }
    }

    fn fetch_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        start_indent: usize,
        init_indent: usize,
    ) {
        let mut allow_minus = false;
        let mut first_line_block = !self.curr_state.in_flow_collection();

        let mut num_newlines = 0;
        let mut tokens = vec![ScalarPlain as usize];
        let mut new_state = match self.curr_state {
            BlockMapKeyExp(ind) => Some(BlockMapValExp(ind)),
            BlockMapValExp(ind) => Some(BlockMap(ind)),
            BlockMap(ind) => Some(BlockMapVal(ind)),
            BlockMapVal(ind) => Some(BlockMap(ind)),
            _ => None,
        };
        let mut curr_indent = self.curr_state.get_block_indent(reader.col());
        let mut had_comment = false;

        while !reader.eof() {
            // In explicit key mapping change in indentation is always an error
            if self.curr_state.wrong_exp_indent(curr_indent) && curr_indent != init_indent {
                tokens.push(ErrorToken as usize);
                self.errors.push(ErrorType::MappingExpectedIndent {
                    actual: curr_indent,
                    expected: init_indent,
                });
                break;
            } else if curr_indent < init_indent {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap
                // b) An error outside of block map
                if !matches!(
                    self.curr_state,
                    BlockMap(_) | BlockMapKeyExp(_) | BlockMapValExp(_) | BlockMapVal(_)
                ) {
                    reader.read_line();
                    tokens.push(ErrorToken as usize);
                    self.errors.push(ExpectedIndent {
                        actual: curr_indent,
                        expected: start_indent,
                    });
                }
                break;
            }

            let (start, end) = match reader.read_plain_one_line(
                allow_minus,
                &mut had_comment,
                self.curr_state.in_flow_collection(),
                &mut tokens,
                &mut self.errors,
            ) {
                Some(x) => x,
                None => break,
            };

            reader.skip_space_tab(true);

            let chr = reader.peek_byte_at(0).unwrap_or(b'\0');

            if chr == b':' && first_line_block {
                if curr_indent == init_indent
                    && matches!(self.curr_state, BlockMapVal(x) if init_indent == x as usize)
                {
                    tokens.push(ScalarEnd as usize);
                    tokens.push(ScalarPlain as usize);
                } else if self.curr_state.is_new_block_col(curr_indent) {
                    reader.consume_bytes(1);
                    new_state = Some(BlockMapVal(curr_indent as u32));
                    tokens.insert(0, MappingStart as usize);
                }

                tokens.push(start);
                tokens.push(end);
                break;
            } else if chr == b':'
                && matches!(self.curr_state, BlockMapValExp(ind) if ind as usize == curr_indent)
            {
                tokens.push(ScalarPlain as usize);
                tokens.push(start);
                tokens.push(end);
                break;
            }

            match num_newlines {
                x if x == 1 => {
                    tokens.push(NewLine as usize);
                    tokens.push(0);
                }
                x if x > 1 => {
                    tokens.push(NewLine as usize);
                    tokens.push(x as usize);
                }
                _ => {}
            }

            tokens.push(start);
            tokens.push(end);
            first_line_block = false;

            if is_newline(chr) {
                let folded_newline = reader.skip_separation_spaces(false);
                if reader.col() >= self.curr_state.indent(0) as usize {
                    num_newlines = folded_newline as u32;
                }
                curr_indent = reader.col();
            }

            if self.curr_state.in_flow_collection() && is_flow_indicator(chr) {
                break;
            }

            match (reader.peek_byte_at(0), self.curr_state) {
                (Some(b'-'), BlockSeq(ind)) if reader.col() == ind as usize => {
                    reader.consume_bytes(1);
                    tokens.push(ScalarEnd as usize);
                    break;
                }
                (Some(b'-'), BlockSeq(ind)) if reader.col() < ind as usize => {
                    reader.read_line();
                    let err_type = ExpectedIndent {
                        expected: ind as usize,
                        actual: curr_indent,
                    };
                    tokens.push(ErrorToken as usize);
                    self.errors.push(err_type);
                    break;
                }
                (Some(b'-'), BlockSeq(ind)) if reader.col() > ind as usize => {
                    allow_minus = true;
                }
                (Some(b':'), BlockMapValExp(ind)) if reader.col() == ind as usize => {
                    break;
                }
                _ => {}
            }
        }

        match new_state {
            Some(BlockMapValExp(x)) => self.curr_state = BlockMapValExp(x),
            Some(BlockMapVal(x)) | Some(BlockMap(x)) if x as usize == start_indent => {
                self.curr_state = new_state.unwrap();
            }
            Some(state) => self.push_state(state),
            None => {}
        }
        self.tokens.extend(tokens);
    }

    fn fetch_explicit_map<B, R: Reader<B>>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart as usize);
        }

        if !reader.peek_byte_at_check(1, is_white_tab_or_break) {
            self.fetch_plain_scalar(reader, reader.col(), reader.col());
        } else {
            reader.consume_bytes(1);
            reader.skip_space_tab(true);
        }
    }

    fn process_map_key<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);

        if self.is_key() {
            self.curr_state = FlowMap(indent as u32);
            self.tokens.push_back(ScalarEnd as usize);
        } else {
            self.fetch_plain_scalar(reader, indent, indent);
        }
    }

    #[inline]
    fn is_prev_sequence(&self) -> bool {
        match self.stack.last() {
            Some(FlowSeq(_)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_map(&self) -> bool {
        match self.curr_state {
            FlowMap(_) | FlowKey(_) | FlowKeyExp(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_key(&self) -> bool {
        match self.curr_state {
            FlowKey(_) | FlowKeyExp(_) => true,
            _ => false,
        }
    }
}

const DOC_END: usize = usize::MAX;
const DOC_START: usize = usize::MAX - 1;
const MAP_END: usize = usize::MAX - 2;
const MAP_START: usize = usize::MAX - 3;
const SEQ_END: usize = usize::MAX - 4;
const SEQ_START: usize = usize::MAX - 5;
const SCALAR_PLAIN: usize = usize::MAX - 7;
const SCALAR_FOLD: usize = usize::MAX - 8;
const SCALAR_LIT: usize = usize::MAX - 9;
const SCALAR_QUOTE: usize = usize::MAX - 10;
const SCALAR_DQUOTE: usize = usize::MAX - 11;
const SCALAR_END: usize = usize::MAX - 12;
const TAG_START: usize = usize::MAX - 13;
const ANCHOR: usize = usize::MAX - 14;
const ALIAS: usize = usize::MAX - 15;
const DIR_RES: usize = usize::MAX - 16;
const DIR_TAG: usize = usize::MAX - 17;
const DIR_YAML: usize = usize::MAX - 18;
const ERROR: usize = usize::MAX - 19;
const NEWLINE: usize = usize::MAX - 20;

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(clippy::enum_clike_unportable_variant)] //false positive see https://github.com/rust-lang/rust-clippy/issues/8043
///
/// [LexerToken] used to Lex YAML files
pub enum LexerToken {
    /// Denotes that value is a [usize] less than [NewLine] and thus its meaning decided by previous Tokens
    /// usually marks a start/end token.
    Mark,
    /// Denotes a newline and must be followed by a [Mark]. If next Mark is 0, it's space otherwise it's a `n`
    /// number of newlines `\n`
    NewLine = NEWLINE,
    /// Error in stream, check [Lexer.errors] for details
    ErrorToken = ERROR,
    /// Directive Tag denoted by `%TAG` and followed by two [Mark] tokens
    DirectiveTag = DIR_TAG,
    /// Directive Tag denoted by `@value` and followed by two [Mark] tokens
    DirectiveReserved = DIR_RES,
    /// YAML directive showing minor/major version of e.g.
    /// ```yaml
    ///     %YAML 1.2
    /// ```
    DirectiveYaml = DIR_YAML,
    /// Plain Scalar that's neither quoted or literal or folded
    /// ```yaml
    ///     example: plain_scalar
    /// ```
    ScalarPlain = SCALAR_PLAIN,
    /// Helper token to end token
    ScalarEnd = SCALAR_END,
    /// Folded scalar token
    /// ```yaml
    ///     example: >
    ///         folded_scalar
    /// ```
    ScalarFold = SCALAR_FOLD,
    /// Literal scalar token
    /// ```yaml
    ///     example: |
    ///         literal_scalar
    /// ```
    ScalarLit = SCALAR_LIT,
    /// Single quoted scalar
    /// ```yaml
    ///     example: 'single quote scalar'
    /// ```
    ScalarSingleQuote = SCALAR_QUOTE,
    /// Double quoted scalar
    /// ```yaml
    ///     example: "double quote scalar"
    /// ```
    ScalarDoubleQuote = SCALAR_DQUOTE,
    /// Element with alternative name e.g. `&foo [x,y]`
    AliasToken = ALIAS,
    /// Reference to an element with alternative name e.g. `*foo`
    AnchorToken = ANCHOR,
    TagStart = TAG_START,
    /// Start of a sequence token, e.g. `[` in 
    /// ```yaml
    ///  [a, b, c]
    /// #^-- start of sequence 
    /// ```
    SequenceStart = SEQ_START,
    /// End of a sequence token, e.g. `]` in 
    /// ```yaml
    ///  [a, b, c]
    /// #        ^-- end of sequence 
    /// ```
    SequenceEnd = SEQ_END,
    /// Start of a map  token, e.g. `{` in 
    /// ```yaml
    ///  { a: b,}
    /// #^-- start of mapping 
    /// ```
    MappingStart = MAP_START,
    /// End of a map  token, e.g. `}` in 
    /// ```yaml
    ///  { a: b}
    /// #      ^-- start of mapping 
    /// ```
    MappingEnd = MAP_END,
    /// Start of document implicit or otherwise
    DocumentStart = DOC_START,
    /// End of document implicit or otherwise
    DocumentEnd = DOC_END,
}

impl LexerToken {

    ///
    /// This method transforms a [LexerToken] into a [DirectiveType]
    /// 
    /// It's UB to call on any [LexexToken] that isn't [DirectiveTag], [DirectiveYaml], or  [DirectiveReserved].
    #[inline(always)]
    pub(crate) unsafe fn to_yaml_directive(self) -> DirectiveType {
        match self {
            DirectiveTag => DirectiveType::Tag,
            DirectiveYaml => DirectiveType::Yaml,
            DirectiveReserved => DirectiveType::Reserved,
            _ => unreachable_unchecked(),
        }
    }

    ///
    /// This method transforms a [LexerToken] into a [ScalarType]
    /// 
    /// It's UB to call on any [LexexToken] that isn't [ScalarPlain], [Mark], [ScalarFold], [ScalarLit], 
    /// [ScalarSingleQuote], [ScalarDoubleQuote].
    #[inline(always)]
    pub(crate) unsafe fn to_scalar(self) -> ScalarType {
        match self {
            ScalarPlain | Mark => ScalarType::Plain,
            ScalarFold => ScalarType::Folded,
            ScalarLit => ScalarType::Literal,
            ScalarSingleQuote => ScalarType::SingleQuote,
            ScalarDoubleQuote => ScalarType::DoubleQuote,
            _ => unreachable_unchecked(),
        }
    }
}

impl From<usize> for LexerToken {
    fn from(value: usize) -> Self {
        pub use LexerToken::*;

        match value {
            DOC_END => DocumentEnd,
            DOC_START => DocumentStart,
            MAP_END => MappingEnd,
            MAP_START => MappingStart,
            SEQ_END => SequenceEnd,
            SEQ_START => SequenceStart,
            SCALAR_PLAIN => ScalarPlain,
            SCALAR_END => ScalarEnd,
            SCALAR_FOLD => ScalarFold,
            SCALAR_LIT => ScalarLit,
            SCALAR_QUOTE => ScalarSingleQuote,
            SCALAR_DQUOTE => ScalarDoubleQuote,
            TAG_START => TagStart,
            ANCHOR => AnchorToken,
            ALIAS => AliasToken,
            DIR_RES => DirectiveReserved,
            DIR_TAG => DirectiveTag,
            DIR_YAML => DirectiveYaml,
            NEWLINE => NewLine,
            ERROR => ErrorToken,
            _ => Mark,
        }
    }
}

impl From<&usize> for LexerToken {
    fn from(value: &usize) -> Self {
        LexerToken::from(*value)
    }
}
