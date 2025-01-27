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

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;
use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::error::Error;
use crate::impls::{AvxScanner, NativeScanner};
use crate::tokenizer::stage1::{NextFn, Stage1Scanner, YamlChunkState};
use crate::tokenizer::visitor::{EventStringVisitor, YamlVisitor};
use crate::util::{ChunkyIterator, NoopValidator};
use crate::SIMD_CHUNK_LENGTH;

pub type ParseResult<T> = Result<T, Error>;

pub struct Parser<'de> {
    idx: usize,
    _data: &'de PhantomData<()>,
}

pub trait Buffer {}

#[derive(Default)]
pub struct Buffers {
    string_buffer: Vec<u8>,
    structural_indexes: Vec<u32>,
}

impl Buffer for Buffers {}

/// Represents the state of the YAML parser.
///
/// This struct is used internally to keep track of various aspects of the parser's state
/// as it processes a YAML document.
///
/// # Fields (for internal use only)
///
/// - `last_indent`: The indentation level of the last parsed line.
/// - `last_col`: The column number of the last parsed character.
/// - `last_row`: The row number of the last parsed character.
/// - `is_prev_double_quotes`: Indicates whether the previous character was a double quote.
/// - `is_prev_iter_odd_single_quote`: Indicates whether the previous iteration ended with an odd number of single quotes.
/// - `is_indent_frozen`: Indicates whether the current indentation level is frozen (cannot be changed).
/// - `is_previous_white_space`: Indicates whether the previous character was whitespace.
/// - `prev_iter_inside_quote`: A bitmask indicating whether each character in the previous chunk was inside quotes.
/// - `is_in_comment`: Indicates whether the parser is currently inside a comment.
#[derive(Default)]
pub struct YamlParserState {
    pub(crate) last_indent: u32,
    pub(crate) last_col: u32,
    pub(crate) last_row: u32,
    pub(crate) is_prev_double_quotes: bool,
    pub(crate) is_prev_iter_odd_single_quote: bool,
    pub(crate) is_indent_frozen: bool,
    pub(crate) is_previous_white_space: bool,
    pub(crate) prev_iter_inside_quote: u64,
    pub(crate) is_in_comment: bool,
}

impl YamlParserState {
    pub(crate) fn merge_state<T: Buffer>(
        &mut self,
        chunk: &[u8; SIMD_CHUNK_LENGTH],
        buffers: &mut T,
        block_state: &mut YamlChunkState,
    ) -> ParseResult<YamlChunkState> {
        todo!()
    }
}

impl YamlParserState {
    pub(crate) fn process_chunk<B: Buffer>(
        &mut self,
        p0: &B,
        p1: YamlChunkState,
    ) -> ParseResult<YamlChunkState> {
        todo!()
    }
}

/// Function that returns right validator for the right architecture
///
/// # Arguments
///
/// * `pre_checked`: `true` when working with [core::str] thus not requiring any validation, `false`
///   otherwise. **Note:** if your [core::str] isn't UTF-8 formatted this will cause Undefined behavior.
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
    /// i.e. don't call Neon
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

impl<'de> Parser<'de> {
    pub fn build_events(input: &'de [u8], hint: Option<usize>) -> String {
        let mut event_visitor = EventStringVisitor::new_with_hint(hint);
        let mut buffer = Buffers::default();

        let mut validator = get_validator(true);

        Self::run_to_end::<Buffers, EventStringVisitor>(
            input,
            &mut event_visitor,
            &mut buffer,
            &mut validator,
        );
        event_visitor.buffer
    }

    fn run_to_end<B: Buffer, V: YamlVisitor<'de>>(
        input: &'de [u8],
        event_visitor: &mut V,
        buffer: &mut B,
        validator: &mut Box<dyn ChunkedUtf8Validator>,
    ) -> Result<(), ()> {
        let mut iter = ChunkyIterator::from_bytes(input);
        let mut state = YamlParserState::default();
        let next_fn = get_stage1_next::<B>();

        // SIMD-ified part
        for chunk in &mut iter {
            let res: Result<YamlChunkState, Error> = unsafe {
                validator.update_from_chunks(chunk);
                next_fn(chunk, buffer, &mut state)
            };
            match res {
                Err(e) => {
                    event_visitor.visit_error(e);
                    return Err(());
                }
                Ok(chunk_state) => state.process_chunk(buffer, chunk_state),
            };
        }

        // Remaining part
        for _rem in iter.finalize() {}

        Ok(())
    }
}
