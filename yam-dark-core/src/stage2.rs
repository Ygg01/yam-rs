// MIT License
//
// Copyright (c) 2024 Simdjson developers
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

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::error::{Error, ErrorType};
use crate::safer_unchecked::GetSaferUnchecked;
use crate::stage1::Stage1Parse;
use crate::{impls, SIMD_INPUT_LENGTH, SIMD_JSON_PADDING};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Parser<'de> {
    idx: usize,
    _data: &'de PhantomData<()>,
}

pub enum StackState {}

pub struct Buffers {
    string_buffer: Vec<u8>,
    yaml_indexes: YamlIndexes,
    stage2_stack: Vec<StackState>,
    simd_input_buffer: AlignedBuf,
}

pub struct YamlIndexes {
    structural_indexes: Vec<u64>,
    indent: Vec<u32>,
    rows: Vec<u32>,
}

impl YamlIndexes {
    pub(crate) fn reserve(&self, p0: usize) {
        todo!()
    }

    pub(crate) fn is_empty(&self) -> bool {
        todo!()
    }
}

impl YamlIndexes {
    pub(crate) fn clear(&self) {
        todo!()
    }
}

/// SIMD aligned buffer
struct AlignedBuf {
    layout: Layout,
    capacity: usize,
    len: usize,
    inner: NonNull<u8>,
}

// We use allow Sync + Send here since we know u8 is sync and send
// we never reallocate or grow this buffer only allocate it in
// create then deallocate it in drop.
//
// An example of this can be found [in the official rust docs](https://doc.rust-lang.org/nomicon/vec/vec-raw.html).

unsafe impl Send for AlignedBuf {}

unsafe impl Sync for AlignedBuf {}

impl AlignedBuf {
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    fn capacity_overflow() -> ! {
        panic!("capacity overflow");
    }

    /// Creates a new buffer that is  aligned with the simd register size
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let layout = match Layout::from_size_align(capacity, SIMD_JSON_PADDING) {
            Ok(layout) => layout,
            Err(_) => Self::capacity_overflow(),
        };
        if mem::size_of::<usize>() < 8 && capacity > isize::MAX as usize {
            Self::capacity_overflow()
        }
        let inner = match unsafe { NonNull::new(alloc(layout)) } {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };
        Self {
            layout,
            capacity,
            len: 0,
            inner,
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.inner.as_ptr()
    }

    unsafe fn set_len(&mut self, n: usize) {
        assert!(
            n <= self.capacity,
            "New size ({}) can not be larger then capacity ({}).",
            n,
            self.capacity
        );
        self.len = n;
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.inner.as_ptr(), self.layout);
        }
    }
}

impl Deref for AlignedBuf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.inner.as_ptr(), self.len) }
    }
}

impl DerefMut for AlignedBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.inner.as_ptr(), self.len) }
    }
}

impl<'de> Parser<'de> {
    fn emit_events(
        input: &'de mut [u8],
        buffer: &mut Buffers,
        event_list: &mut String,
    ) -> Result<()> {
        const LOTS_OF_ZEROES: [u8; 64] = [0u8; SIMD_INPUT_LENGTH];
        let len = input.len();
        let simd_safe_len = len + SIMD_INPUT_LENGTH;

        buffer.string_buffer.clear();
        buffer.string_buffer.reserve(len + SIMD_JSON_PADDING);

        let input_buffer = &mut buffer.simd_input_buffer;
        if input_buffer.capacity() < simd_safe_len {
            *input_buffer = AlignedBuf::with_capacity(simd_safe_len)
        }

        unsafe {
            input_buffer
                .as_mut_ptr()
                .copy_from_nonoverlapping(input.as_ptr(), len);

            // initialize all remaining bytes
            // this also ensures we have a 0 to terminate the buffer
            input_buffer
                .as_mut_ptr()
                .add(len)
                .copy_from_nonoverlapping(LOTS_OF_ZEROES.as_ptr(), SIMD_INPUT_LENGTH);

            // safety: all bytes are initialized
            input_buffer.set_len(simd_safe_len);

            Self::find_structural_bits(input, &mut buffer.yaml_indexes).map_err(Error::generic)?;
        };

        Self::build_tape(
            input,
            input_buffer,
            &mut buffer.string_buffer,
            &buffer.yaml_indexes,
            &mut buffer.stage2_stack,
            event_list,
        )
    }

    pub(crate) fn build_tape(
        input: &'de mut [u8],
        input2: &[u8],
        buffer: &mut [u8],
        yaml_indexes: &YamlIndexes,
        stack: &mut Vec<StackState>,
        res: &mut String,
    ) -> Result<()> {
        todo!()
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    pub(crate) unsafe fn find_structural_bits(
        input: &mut [u8],
        structural_indexes: &mut YamlIndexes,
    ) -> std::result::Result<(), ErrorType> {
        unsafe {
            let x = {
                if std::is_x86_feature_detected!("avx2") {
                    Parser::_find_structural_bits::<impls::avx2::SimdInput>(
                        input,
                        structural_indexes,
                    )
                } else if std::is_x86_feature_detected!("sse4.2") {
                    Parser::_find_structural_bits::<impls::sse42::SimdInput>(
                        input,
                        structural_indexes,
                    )
                } else {
                    Parser::_find_structural_bits::<impls::native::SimdInput>(
                        input,
                        structural_indexes,
                    )
                }
            };
            x
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) unsafe fn _find_structural_bits<S: Stage1Parse>(
        input: &[u8],
        structural_indexes: &mut YamlIndexes,
    ) -> std::result::Result<(), ErrorType> {
        let len = input.len();
        let len = input.len();
        // 8 is a heuristic number to estimate it turns out a rate of 1/8 structural characters
        // leads almost never to relocations.
        structural_indexes.clear();
        structural_indexes.reserve(len / 8);

        let mut utf8_validator = S::Utf8Validator::new();

        // we have padded the input out to 64 byte multiple with the remainder being
        // zeros

        // persistent state across loop
        // does the last iteration end with an odd-length sequence of backslashes?
        // either 0 or 1, but a 64-bit value
        let mut prev_iter_ends_odd_backslash: u64 = 0;
        // does the previous iteration end inside a double-quote pair?
        let mut prev_iter_inside_quote: u64 = 0;
        // either all zeros or all ones
        // does the previous iteration end on something that is a predecessor of a
        // pseudo-structural character - i.e. whitespace or a structural character
        // effectively the very first char is considered to follow "whitespace" for
        // the
        // purposes of pseudo-structural character detection so we initialize to 1
        let mut prev_iter_ends_pseudo_pred: u64 = 1;

        // structurals are persistent state across loop as we flatten them on the
        // subsequent iteration into our array pointed to be base_ptr.
        // This is harmless on the first iteration as structurals==0
        // and is done for performance reasons; we can hide some of the latency of the
        // expensive carryless multiply in the previous step with this work
        let mut structurals: u64 = 0;

        let lenminus64: usize = if len < 64 { 0 } else { len - 64 };
        let mut idx: usize = 0;
        let mut error_mask: u64 = 0; // for unescaped characters within strings (ASCII code points < 0x20)

        while idx < lenminus64 {
            /*
            #ifndef _MSC_VER
              __builtin_prefetch(buf + idx + 128);
            #endif
             */
            let chunk = input.get_kinda_unchecked(idx..idx + 64);
            utf8_validator.update_from_chunks(chunk);

            let input = S::new(chunk);
            // detect odd sequences of backslashes
            let odd_ends: u64 =
                input.find_odd_backslash_sequences(&mut prev_iter_ends_odd_backslash);

            // detect insides of quote pairs ("quote_mask") and also our quote_bits
            // themselves
            let mut quote_bits: u64 = 0;
            let quote_mask: u64 = input.find_quote_mask_and_bits(
                odd_ends,
                &mut prev_iter_inside_quote,
                &mut quote_bits,
                &mut error_mask,
            );

            // take the previous iterations structural bits, not our current iteration,
            // and flatten
            S::flatten_bits(structural_indexes, idx as u32, structurals);

            let mut whitespace: u64 = 0;
            input.find_whitespace_and_structurals(&mut whitespace, &mut structurals);

            // fixup structurals to reflect quotes and add pseudo-structural characters
            structurals = S::finalize_structurals(
                structurals,
                whitespace,
                quote_mask,
                quote_bits,
                &mut prev_iter_ends_pseudo_pred,
            );
            idx += SIMD_INPUT_LENGTH;
        }

        // we use a giant copy-paste which is ugly.
        // but otherwise the string needs to be properly padded or else we
        // risk invalidating the UTF-8 checks.
        if idx < len {
            let mut tmpbuf: [u8; SIMD_INPUT_LENGTH] = [0x20; SIMD_INPUT_LENGTH];
            tmpbuf
                .as_mut_ptr()
                .copy_from(input.as_ptr().add(idx), len - idx);
            utf8_validator.update_from_chunks(&tmpbuf);

            let input = S::new(&tmpbuf);

            // detect odd sequences of backslashes
            let odd_ends: u64 =
                input.find_odd_backslash_sequences(&mut prev_iter_ends_odd_backslash);

            // detect insides of quote pairs ("quote_mask") and also our quote_bits
            // themselves
            let mut quote_bits: u64 = 0;
            let quote_mask: u64 = input.find_quote_mask_and_bits(
                odd_ends,
                &mut prev_iter_inside_quote,
                &mut quote_bits,
                &mut error_mask,
            );

            // take the previous iterations structural bits, not our current iteration,
            // and flatten
            S::flatten_bits(structural_indexes, idx as u32, structurals);

            let mut whitespace: u64 = 0;
            input.find_whitespace_and_structurals(&mut whitespace, &mut structurals);

            // fixup structurals to reflect quotes and add pseudo-structural characters
            structurals = S::finalize_structurals(
                structurals,
                whitespace,
                quote_mask,
                quote_bits,
                &mut prev_iter_ends_pseudo_pred,
            );
            idx += SIMD_INPUT_LENGTH;
        }
        // This test isn't in upstream, for some reason the error mask is et for then.
        if prev_iter_inside_quote != 0 {
            return Err(ErrorType::Syntax);
        }
        // finally, flatten out the remaining structurals from the last iteration
        S::flatten_bits(structural_indexes, idx as u32, structurals);

        // a valid JSON file cannot have zero structural indexes - we should have
        // found something (note that we compare to 1 as we always add the root!)
        if structural_indexes.is_empty() {
            return Err(ErrorType::Eof);
        }

        if error_mask != 0 {
            return Err(ErrorType::Syntax);
        }

        if utf8_validator.finalize(None).is_err() {
            Err(ErrorType::InvalidUtf8)
        } else {
            Ok(())
        }
    }
}
