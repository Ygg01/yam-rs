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

use std::alloc::{alloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

use crate::{SIMD_INPUT_LENGTH, SIMD_JSON_PADDING};
use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Parser<'de> {
    idx: usize,
    _data: &'de PhantomData<()>,
}

pub enum StackState {}

pub struct Buffers {
    string_buffer: Vec<u8>,
    structural_indexes: Vec<u64>,
    stage2_stack: Vec<StackState>,
    input_buffer: AlignedBuf,
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

        let input_buffer = &mut buffer.input_buffer;
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

            Self::find_structural_bits(input, &mut buffer.structural_indexes)
                .map_err(Error::generic)?;
        };

        Self::build_tape(
            input,
            input_buffer,
            &mut buffer.string_buffer,
            &buffer.structural_indexes,
            &mut buffer.stage2_stack,
            event_list,
        )
    }

    pub (crate) fn build_tape(
        input: &'de mut [u8],
        input2: &[u8],
        buffer: &mut [u8],
        structural_indexes: &[u32],
        stack: &mut Vec<StackState>,
        res: &mut String,
    ) -> Result<()> {

    }
    pub(crate) unsafe fn find_structural_bits(p0: &mut [u8], p1: &mut Vec<u64>) -> _ {
        todo!()
    }
}