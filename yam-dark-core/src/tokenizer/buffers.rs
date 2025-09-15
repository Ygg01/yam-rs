use core::slice::SliceIndex;
use yam_common::Mark;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer {
    /// Get the underlying buffer.
    fn append<S>(&mut self, src: &S, mark: &mut Mark);
    fn reserve(&mut self, len: usize);
}

pub trait Indexer {}

impl YamlBuffer for () {
    fn append<'s, S>(&mut self, _src: &S, _mark: &mut Mark) {}

    fn reserve(&mut self, len: usize) {}
}

/// Traits for input sources used in yam.rs
///
/// It allows abstracting over input strings and buffers
pub trait YamlSource<'s> {
    unsafe fn get_span_unsafely<T: SliceIndex<[u8], Output = [u8]>>(&self, span: T) -> &'s [u8];
    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8;
    fn has_more(&self) -> bool;
    fn get_len(&self) -> usize;
    fn get_bytes(&self) -> &[u8];
}

impl<'s> YamlSource<'s> for &'s [u8] {
    unsafe fn get_span_unsafely<T: SliceIndex<[u8], Output = [u8]>>(&self, span: T) -> &'s [u8] {
        self.get_unchecked(span)
    }

    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8 {
        *self.get_unchecked(pos)
    }

    fn has_more(&self) -> bool {
        false
    }

    fn get_len(&self) -> usize {
        self.len()
    }

    fn get_bytes(&self) -> &[u8] {
        self
    }
}

impl<'s> YamlSource<'s> for &'s str {
    unsafe fn get_span_unsafely<T: SliceIndex<[u8], Output = [u8]>>(&self, span: T) -> &'s [u8] {
        self.as_bytes().get_unchecked(span)
    }

    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8 {
        *self.as_bytes().get_unchecked(pos)
    }

    fn has_more(&self) -> bool {
        false
    }

    fn get_len(&self) -> usize {
        self.len()
    }

    fn get_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
