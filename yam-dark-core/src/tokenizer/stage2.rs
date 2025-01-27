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

trait YamlIndex {}

#[doc(hidden)]
#[derive(Default)]
pub struct YamlParserState {
    pub(crate) prev_iter_ends_pseudo_pred: u64,
    pub(crate) prev_iter_ends_odd_backslash: u64,
    pub(crate) prev_iter_inside_quote: u64,
    pub(crate) prev_iter_odd_backslash: u32,
    pub(crate) prev_iter_odd_quote: u32,
    pub(crate) last_indent: u32,
    pub(crate) last_col: u32,
    pub(crate) last_row: u32,
    pub(crate) is_indent_frozen: bool,
}

impl YamlParserState {
    pub(crate) fn merge_state<T: Buffer>(
        &mut self,
        chunk: &[u8; 64],
        buffers: &mut T,
        block_state: &mut YamlChunkState,
    ) -> ParseResult<YamlChunkState> {
        todo!()
    }
}

impl YamlParserState {
    pub(crate) fn process_chunk<B: Buffer>(&mut self, p0: &B, p1: YamlChunkState) {
        todo!()
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn get_validator(pre_checked: bool) -> Box<dyn ChunkedUtf8Validator> {
    if pre_checked {
        unsafe {
            // Is always safe for preformatted utf8
            return Box::new(NoopValidator::new());
        }
    }

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

        // SIMDified part
        for chunk in &mut iter {
            unsafe {
                validator.update_from_chunks(chunk);
                let res: Result<YamlChunkState, Error> = next_fn(chunk, buffer, &mut state);
                let block = match res {
                    Err(e) => {
                        event_visitor.visit_error(e);
                        return Err(());
                    }
                    Ok(x) => x,
                };
                state.process_chunk(buffer, block);
            }
        }

        // Remaining part
        for _rem in iter.finalize() {}

        Ok(())
    }
}
