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

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::error::Error;

pub type ParseResult<T> = Result<T, Error>;

pub struct Parser<'de> {
    idx: usize,
    _data: &'de PhantomData<()>,
}


pub trait Buffer {
    
}

pub struct Buffers {
    string_buffer: Vec<u8>,
    structural_indexes: Vec<u32>,
}

impl Buffer for Buffers {}


trait YamlIndex {

}

pub(crate) struct YamlParserState {

}

impl<'de> Parser<'de> {
    pub fn build_events(
        input: &[u8],
        hint: Option<usize>,
    ) -> String {
        // TODO
        let buff = String::with_capacity(hint.unwrap_or(100));
        buff
    }

}

