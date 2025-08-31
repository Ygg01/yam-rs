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
use crate::tokenizer::buffers::{YamlBuffer, YamlSource};
use crate::tokenizer::parser::ChunkState;
use crate::tokenizer::stage1::Stage1Scanner;
use crate::util::{str_to_chunk, ChunkArrayIter};
use crate::{ChunkyIterWrap, EventListener, YamlStructurals};
use crate::{YamlError, YamlResult};
use alloc::vec;
use yam_common::Mark;

pub type ParseResult<T> = Result<T, YamlError>;

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
#[doc(hidden)]
/// TODO docs and Safety guarantees
pub unsafe trait Stage2Scanner {
    fn parse_double_quote(input: &[u8], state: YamlStructurals) -> Mark {
        Mark { start: 0, end: 0 }
    }
    fn parse_single_quote(input: &[u8; 64]) -> Mark {
        Mark { start: 0, end: 0 }
    }
    fn parse_block_string(input: &[u8], state: YamlStructurals) -> Mark {
        Mark { start: 0, end: 0 }
    }
    fn parse_unquoted(input: &[u8], state: YamlStructurals) -> Mark {
        Mark { start: 0, end: 0 }
    }
}

#[inline]
pub(crate) fn get_fast_double_quote<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
    source: &S,
    buffer: &mut B,
    indent: i64,
    event_listener: &mut E,
) -> YamlResult<()> {
    fn run_double_quote_inner<
        's,
        A: Stage2Scanner,
        S: YamlSource<'s>,
        B: YamlBuffer,
        E: EventListener,
    >() -> YamlResult<()> {
        //TODO
        Ok(())
    }

    // #[cfg(target_arch = "x86_64")]
    // {
    //     if is_x86_feature_detected!("avx2") {
    //         // SAFETY: We have detected the feature is enabled at runtime,
    //         // so it's safe to call this function.
    //         return fill_tape_inner::<AvxScanner, NoopValidator>(input.as_bytes(), state);
    //     }
    // }
    run_double_quote_inner::<NativeScanner, S, B, E>()
}

#[inline]
pub(crate) fn get_fast_single_quote<'s, YS: YamlSource<'s>, YB: YamlBuffer, EL: EventListener>(
    source: &YS,
    buffer: &mut YB,
    event_listener: &mut EL,
    state: &mut YamlStructurals,
) -> YamlResult<()> {
    // #[cfg(target_arch = "x86_64")]
    // {
    //     if is_x86_feature_detected!("avx2") {
    //         // SAFETY: We have detected the feature is enabled at runtime,
    //         // so it's safe to call this function.
    //         return fill_tape_inner::<AvxScanner, NoopValidator>(input.as_bytes(), state);
    //     }
    // }
    run_single_quote_inner::<NativeScanner, YS, YB, EL>(source, buffer, event_listener, state)
}
#[inline]
fn run_single_quote_inner<
    's,
    S2: Stage2Scanner,
    YS: YamlSource<'s>,
    YB: YamlBuffer,
    EL: EventListener,
>(
    source: &YS,
    buffer: &mut YB,
    event_listener: &mut EL,
    yaml_structurals: &mut YamlStructurals,
) -> YamlResult<()> {
    // SAFETY: The Stage1Scanner must always return a correct index within the code.
    let mut chunk_iter = ChunkArrayIter::from_bytes(unsafe {
        source.get_span_unsafely(Mark {
            start: yaml_structurals.idx,
            end: yaml_structurals.next_idx(),
        })
    });
    for x in chunk_iter.by_ref() {
        // S2::parse_single_quote()
    }

    todo!()
}

// #[inline]
// pub(crate) fn get_fast_block_scalar<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
//     source: &S,
//     buffer: &mut B,
//     indent: i64,
//     event_listener: &mut E,
// ) -> YamlResult<()> {
//     Ok(())
// }
//
// #[inline]
// pub(crate) fn get_fast_unquoted_scalar<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
//     source: &S,
//     buffer: &mut B,
//     indent: i64,
//     event_listener: &mut E,
// ) -> YamlResult<()> {
//     Ok(())
// }

#[test]
fn test_parsing_basic_processing1() {
    let input = r#"
        "test"
    "#;
    let mut chunk_iter_state = ChunkState::default();
    let mut state = YamlStructurals::default();
    let wrap = str_to_chunk(input);
    let mut chunk_iter = ChunkyIterWrap::from_bytes(&wrap);

    let chunk = chunk_iter.next().expect("Expected a chunk");
    let chunk_state = NativeScanner::next(chunk, &mut chunk_iter_state, &mut state, &mut 0);
    state.process_chunk::<NativeScanner>(&chunk_state);

    let expected_structurals = vec![9usize];
    assert_eq!(expected_structurals, state.structurals);
}
