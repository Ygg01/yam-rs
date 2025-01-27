#![allow(unused)]

// MIT License
//
// Copyright (c) 2024 Ygg One
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::impls::{AvxScanner, NativeScanner};
use crate::tape::Node;
use crate::tokenizer::stage1::{NextFn, Stage1Scanner};
use crate::util::NoopValidator;
use crate::{ChunkyIterator, YamlChunkState};
use crate::{YamlError, YamlResult};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub type ParseResult<T> = Result<T, YamlError>;

pub struct Deserializer<'de> {
    idx: usize,
    tape: Vec<Node<'de>>,
    _data: &'de PhantomData<()>,
}

pub trait Buffer {
    fn get_buffer(&self) -> &[u8];
    unsafe fn get_byte_unsafely(&self, pos: usize) -> u8 {
        *self.get_buffer().get_unchecked(pos)
    }
}

#[derive(Default)]
pub struct OwnedBuffer {
    string_buffer: Vec<u8>,
}

#[derive(Default)]
pub struct BorrowBuffer<'buff> {
    string_buffer: &'buff [u8],
}

impl Buffer for OwnedBuffer {
    fn get_buffer(&self) -> &[u8] {
        self.string_buffer.as_slice()
    }
}
impl<'b> Buffer for BorrowBuffer<'b> {
    fn get_buffer(&self) -> &[u8] {
        self.string_buffer
    }
}

impl<'a> BorrowBuffer<'a> {
    fn new(string_buffer: &'a str) -> Self {
        Self {
            string_buffer: string_buffer.as_bytes(),
        }
    }
}

fn fill_tape<'de, B: Buffer>(
    input: &'de [u8],
    buffer: &mut B,
    tape: &mut [Node<'de>],
) -> ParseResult<()> {
    Deserializer::fill_tape::<B, NativeScanner>(input, buffer, tape)
}

#[derive(Debug, Default)]
pub(crate) enum State {
    #[default]
    PreDocStart,
    AfterDocBlock,
    InDocEnd,
    FlowSeq,
    FlowMap,
    DocBlock,
    BlockSeq,
    BlockMap,
}

impl<'de> Deserializer<'de> {
    fn fill_tape<B: Buffer, S: Stage1Scanner>(
        input: &'de [u8],
        buffer: &mut B,
        tape: &mut [Node<'de>],
    ) -> YamlResult<()> {
        let mut iter = ChunkyIterator::from_bytes(input);
        let mut state = YamlParserState::default();
        let mut validator = get_validator(false);
        let mut indent_info = YamlIndentInfo::default();

        let next_fn = get_stage1_next::<B>();

        for chunk in iter {
            // SAFETY: The get_validator function should return the correct validator for any given
            // CPU architecture.
            // PANIC safe: the chunk is always 64 characters long
            unsafe {
                validator.update_from_chunks(chunk);
            }

            // SAFETY: The next_fn should return the correct function for any given CPU
            let chunk_state: YamlChunkState = S::next(chunk, buffer, &mut state);
            state.process_chunk::<B, S>(buffer, &chunk_state, &mut indent_info)?;
        }

        Self::build_tape(&mut state, buffer, tape)
    }

    fn build_tape<B: Buffer>(
        parser_state: &mut YamlParserState,
        buffer: &mut B,
        _tape: &mut [Node],
    ) -> YamlResult<()> {
        let mut idx = 0;
        let mut chr = b' ';

        macro_rules! update_char {
            () => {
                if parser_state.pos < parser_state.structurals.len() {
                    // SAFETY: this method will be safe if YamlParserState structurals are safe
                    let chr = unsafe {
                        buffer.get_byte_unsafely(
                            *parser_state.structurals.get_unchecked(parser_state.pos),
                        )
                    };
                    parser_state.pos += 1;
                    chr
                } else {
                    // Return error and defer to cleanup.
                    break YamlResult::Err(YamlError::UnexpectedEof);
                }
            };
        }

        let result = loop {
            //early bailout
            match parser_state.state {
                State::PreDocStart => {
                    chr = update_char!();
                    match chr {
                        _ => {}
                    }
                }
                _ => {}
            }
        };

        Self::cleanup();

        result
    }

    fn cleanup() {
        todo!()
    }
}

/// Represents the internal state of a YAML parser.
///
/// The `YamlParserState` struct is used to track the parser's state as it processes
/// a YAML document. This state includes various counters and flags needed to
/// correctly parse and understand the structure and content of the document.
///
/// # Fields (for internal use only)
///
/// ## State fields:
/// - `state`: current state of the Parser
///
/// ## Structural fields:
/// - `structurals`: A vector of position indices marking structural elements
///   like start and end positions of nodes in the YAML document.
/// - `byte_cols`: For each structurals byte this its corresponding byte column.
/// - `byte_rows`: For each structurals byte this its corresponding byte row.
/// - `indents`: For each structurals byte this its corresponding indentation.
/// - `pos`: The current  position in the structural array.
///
/// ## Sparse fields:
/// - `open_close_tag`: A list of all structurals that start or end YAML
/// - `potential_block`: A list of structurals that are potentially valid block tokens.
///
/// ## Previous chunk fields
/// - `last_indent`: The indentation level of the last chunk processed.
/// - `last_col`: The column position of the last chunk processed.
/// - `last_row`: The row position of the last chunk processed.
/// - `previous_indent`: The indentation level before the current chunk.
/// - `prev_iter_inside_quote`: Tracks the quoting state of the previous iteration
///   to determine the continuation of strings across lines.
/// - `is_indent_running`: A flag indicating if the parser is currently processing
///   an indentation level.
/// - `is_previous_white_space`: Indicates if the last processed character was a whitespace.
/// - `is_prev_iter_odd_single_quote`: Tracks if there's an odd number of single quotes
///   up to the previous iteration, affecting string parsing.
/// - `is_prev_double_quotes`: Indicates if the string being parsed is inside double quotes.
/// - `is_in_comment`: A flag that tracks if the parser is currently inside a comment segment.
///
/// This struct is part of the internal workings of a YAML parsing library, often
/// utilized by the parsing modules such as `stage1` and `stage2` for processing
/// various stages of parsing a YAML document.

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct YamlParserState {
    // State field
    pub(crate) state: State,

    // Structural fields
    pub(crate) structurals: Vec<usize>,
    pub(crate) byte_cols: Vec<u32>,
    pub(crate) byte_rows: Vec<u32>,
    pub(crate) indents: Vec<u32>,
    pub(crate) pos: usize,

    // Sparse fields
    pub(crate) open_close_tag: Vec<usize>,
    pub(crate) potential_block: Vec<usize>,

    // Previous chunk fields
    pub(crate) last_indent: u32,
    pub(crate) last_col: u32,
    pub(crate) last_row: u32,
    pub(crate) previous_indent: u32,
    pub(crate) prev_iter_inside_quote: u64,
    pub(crate) is_indent_running: bool,
    pub(crate) is_previous_white_space: bool,
    pub(crate) is_prev_iter_odd_single_quote: bool,
    pub(crate) is_prev_double_quotes: bool,
    pub(crate) is_in_comment: bool,
}

/// Transient data about cols, rows and indents
/// 
/// Each u32 corresponds to cols/row/indent triplet for 64 bits of the chunk.
/// On initialization it will be set to 0.
pub struct YamlIndentInfo {
    /// Cols of the chunk
    pub(crate) cols: [u32; 64],
    /// Rows of the chunk
    pub(crate) rows: [u32; 64],
    /// Indents of the chunk
    pub(crate) indents: [u32; 64],

    /// Last row since then
    pub(crate) last_row_mask: u32,
}

impl Default for YamlIndentInfo {
    fn default() -> Self {
        YamlIndentInfo {
            cols: [0; 64],
            rows: [0; 64],
            indents: [0; 64],
            last_row_mask: 0,
        }
    }
}

impl YamlParserState {
    pub(crate) fn process_chunk<B: Buffer, S: Stage1Scanner>(
        &mut self,
        buffer: &mut B,
        chunk_state: &YamlChunkState,
        indent_info: &mut YamlIndentInfo,
    ) -> YamlResult<()> {
        // Then we calculate rows, cols for structurals
        S::calculate_indent_info(self, chunk_state, indent_info);

        // And based on rows/cols for structurals, we calculate indents
        S::calculate_relative_indents(self, chunk_state, indent_info);

        // First we find all interesting structural bits
        S::flatten_bits_yaml(self, chunk_state, indent_info);

        if chunk_state.error_mask == 0 {
            Ok(())
        } else {
            // TODO: Deal with errors
            Err(YamlError::UnexpectedEof)
        }
    }

    pub(crate) fn next_state() -> YamlResult<()> {
        todo!()
    }
}

/// Function that returns right validator for the right architecture
///
/// # Arguments
///
/// * `pre_checked`: `true` when working with [`core::str`] thus not requiring any validation, `false`
///   otherwise. **Note:** if your [`core::str`] isn't UTF-8 formatted this will cause Undefined behavior.
///
/// returns: `Box<dyn ChunkedUtf8Validator, Global>` a heap allocated [`ChunkedUtf8Validator`] that
/// is guaranteed to be correct for your CPU architecture.
///
#[cfg_attr(not(feature = "no-inline"), inline)]
fn get_validator(pre_checked: bool) -> Box<dyn ChunkedUtf8Validator> {
    if pre_checked {
        /// Safety: Always safe for preformatted utf8
        unsafe {
            // Is always safe for preformatted utf8
            return Box::new(NoopValidator::new());
        }
    }

    /// Safety: Only unsafe thing here is from calling right Scanner for right CPU architecture
    /// i.e. don't call Neon on x86 architecture
    unsafe {
        if core_detect::is_x86_feature_detected!("avx2") {
            Box::new(AvxScanner::validator())
        } else {
            Box::new(NativeScanner::validator())
        }
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn get_stage1_next<B: Buffer>() -> NextFn<B> {
    NativeScanner::next::<B>
}

#[test]
fn test_parsing_basic_processing1() {
    let input = r#"""
        test
    """#;
    let mut buffer = BorrowBuffer::new(input);
    let mut state = YamlParserState::default();
    let mut validator = get_validator(false);
    let mut chunk_iter = ChunkyIterator::from_bytes(input.as_bytes());
    let mut info = YamlIndentInfo::default();

    let chunk = chunk_iter.next().expect("Missing chunk!");
    let chunk_state = NativeScanner::next(chunk, &mut buffer, &mut state);
    let res =
        state.process_chunk::<BorrowBuffer, NativeScanner>(&mut buffer, &chunk_state, &mut info);

    let expected_structurals = vec![0usize];
    assert_eq!(expected_structurals, state.structurals);
}
