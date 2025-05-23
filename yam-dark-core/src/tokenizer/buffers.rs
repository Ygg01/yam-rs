use core::slice::SliceIndex;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer<'de> {
    /// Get the underlying buffer.
    fn get_mut_buffer(&mut self) -> &mut Self {
        self
    }

    /// Get the representation of the buffer as a slice.
    fn get_bytes(&self) -> &[u8];
    /// Access position in a buffer without overhead of access
    ///
    /// # Safety
    /// The `pos` argument must be within bound, or this is Undefined Behavior.
    unsafe fn get_byte_unsafely<I: SliceIndex<[u8]>>(&self, pos: usize) -> u8 {
        *self.get_bytes().get_unchecked(pos)
    }

    /// Access position in a buffer without overhead of access
    ///
    /// # Safety
    /// The `start` and `end` arguments must be within bounds, or this is Undefined Behavior.
    unsafe fn get_span_unsafely(&self, start: usize, end: usize) -> &[u8] {
        self.get_bytes().get_unchecked(start..end)
    }
}

#[derive(Default)]
pub struct BorrowBuffer<'buff> {
    string_buffer: &'buff [u8],
}

impl<'de> YamlBuffer<'de> for BorrowBuffer<'de> {
    fn get_bytes(&self) -> &[u8] {
        self.string_buffer
    }
}

impl<'a> BorrowBuffer<'a> {
    pub(crate) fn new(string_buffer: &'a str) -> Self {
        Self {
            string_buffer: string_buffer.as_bytes(),
        }
    }
}
