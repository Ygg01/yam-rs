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
use crate::stage1::{AvxScanner, NativeScanner, NextFn, Stage1Scanner};
use crate::util::NoopValidator;

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

#[derive(Default)]
pub(crate) struct YamlParserState {}

impl<'de> Parser<'de> {
    pub fn build_events(_input: &[u8], hint: Option<usize>) -> String {
        // TODO
        let buff = String::with_capacity(hint.unwrap_or(100));
        let mut buffer = Buffers::default();
        let mut state = YamlParserState::default();

        #[cfg_attr(not(feature = "no-inline"), inline)]
        fn get_stage1_next() -> NextFn<Buffers> {
            if core_detect::is_x86_feature_detected!("avx2") {
                AvxScanner::next::<Buffers>
            } else {
                NativeScanner::next::<Buffers>
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
            if core_detect::is_x86_feature_detected!("avx2") {
                Box::new(AvxScanner::validator())
            } else {
                Box::new(NativeScanner::validator())
            }
        }

        let next_fn = get_stage1_next();
        let mut validator = get_validator(true);
        
        let chunk = _input.chunks_exact(8);
        // chunk.next_chunk()
        
        unsafe {
            validator.update_from_chunks(&[1, 2, 3]);
            let z = next_fn(&[0; 64], &mut buffer, &mut state);
        }

        buff
    }
}
