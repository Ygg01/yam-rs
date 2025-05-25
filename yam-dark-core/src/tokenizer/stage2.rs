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

use crate::impls::NativeScanner;
use crate::tokenizer::buffers::BorrowBuffer;
use crate::tokenizer::get_validator;
use crate::tokenizer::stage1::Stage1Scanner;
use crate::{ChunkyIterator, YamlChunkState};
use crate::{YamlError, YamlResult};
use alloc::vec;
use alloc::vec::Vec;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub type ParseResult<T> = Result<T, YamlError>;

/// Represents the internal state of a YAML parser.
///
/// The `YamlParserState` struct is used to track the parser's state as it processes
/// a YAML document. This state includes various counters and flags needed to
/// correctly parse and understand the structure and content of the document.
///
/// # Fields (for internal use only)
///
/// ## State fields:
/// * `state`: current state of the Parser
///
/// ## Structural fields:
/// * `structurals`: A vector of position indices marking structural elements
///   like start and end positions of nodes in the YAML document.
/// * `byte_cols`: For each structural, a byte has its corresponding byte column.
/// * `byte_rows`: For each structural, a byte has its corresponding byte row.
/// * `indents`: For each structural, a byte has its corresponding indentation.
/// * `pos`: The current position in the structural array.
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
/// - `Prev_iter_inside_quote`: Tracks the quoting state of the previous iteration
///   to determine the continuation of strings across lines.
/// - `is_indent_running`: A flag indicating if the parser is currently processing
///   an indentation level.
/// - `is_previous_white_space`: Indicates if the last processed character was whitespace.
/// - `is_prev_iter_odd_single_quote`: Tracks if there's an odd number of single quotes
///   up to the previous iteration, affecting string parsing.
/// - `is_prev_double_quotes`: Indicates if the string being parsed is inside double quotes.
/// - `is_in_comment`: A flag that tracks if the parser is currently inside a comment segment.
///
/// This struct is part of the internal workings of a YAML parsing library, often
/// used by the parsing modules such as `stage1` and `stage2` for processing
/// various stages of parsing a YAML document.

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct YamlParserState {
    // State field
    pub(crate) state: State,

    /// Structural fields
    pub structurals: Vec<usize>,
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

#[doc(hidden)]
/// Transient data about cols, rows and indents that is valid per chunk
///
/// It will default, [`cols`](field@YamlIndentInfo#cols)/`rows`/`indent` to `[0; 64]` and set [`row_indents_mask`] to zero.
pub struct YamlIndentInfo {
    /// Cols of the chunk, used by structurals to find only used ones
    pub cols: [u32; 64],
    /// Rows of the chunk, used by structurals to find only used ones
    pub rows: [u32; 64],
    /// Indents of each row in chunk they are guaranteed to be less than
    pub indents: [u32; 64],
    /// Mask for extracting indents based on [`YamlIndentInfo::rows`]
    pub row_indent_mask: u32,
}

impl Default for YamlIndentInfo {
    fn default() -> Self {
        YamlIndentInfo {
            cols: [0; 64],
            rows: [0; 64],
            indents: [0; 64],
            row_indent_mask: 0,
        }
    }
}

impl YamlParserState {
    pub(crate) fn process_chunk<'de, S>(&mut self, chunk_state: &YamlChunkState)
    where
        S: Stage1Scanner,
    {
        // Then we calculate rows, cols for structurals
        let mut indent_info = YamlIndentInfo::default();
        S::calculate_row_col_info(chunk_state.characters.line_feeds, self, &mut indent_info);

        // And based on rows/cols for structurals, we calculate indents
        S::calculate_relative_indents(chunk_state, self, &mut indent_info);

        // First, we find all interesting structural bits
        S::flatten_bits_yaml(chunk_state, self, &mut indent_info);
    }

    pub(crate) fn next_state() -> YamlResult<()> {
        todo!()
    }
}

#[test]
fn test_parsing_basic_processing1() {
    let input = r#"
        "test"
    "#;
    let mut buffer = BorrowBuffer::new(input);
    let mut state = YamlParserState::default();
    let mut validator = get_validator::<NativeScanner>(false);
    let mut chunk_iter = ChunkyIterator::from_bytes(input.as_bytes());

    let chunk = chunk_iter.next().expect("Missing chunk!");
    let chunk_state = NativeScanner::next(chunk, &mut state, &mut 0);
    let res = state.process_chunk::<NativeScanner>(&chunk_state);

    let expected_structurals = vec![9usize];
    assert_eq!(expected_structurals, state.structurals);
}
