use crate::tape::Mark;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer {
    /// Get the underlying buffer.
    fn append<S>(&mut self, src: &S, mark: &mut Mark);
}

pub trait YamlSource<'s> {
    unsafe fn get_span_unsafely(&self, span: Mark) -> &'s [u8];
    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8;
}

impl<'s> YamlSource<'s> for &'s [u8] {
    unsafe fn get_span_unsafely(&self, span: Mark) -> &'s [u8] {
        self.get_unchecked(span.start..span.end)
    }

    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8 {
        *self.get_unchecked(pos)
    }
}

impl YamlBuffer for () {
    fn append<'s, S>(&mut self, _src: &S, _mark: &mut Mark) {}
}
