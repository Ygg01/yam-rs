use crate::tape::Mark;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer {
    /// Get the underlying buffer.
    fn append<S>(&mut self, src: &S, mark: &mut Mark);
}

pub trait YamlSource<'s> {
    fn get_span_unsafely(&self, span: Mark) -> &'s [u8];
}

impl<'s> YamlSource<'s> for &'s [u8] {
    fn get_span_unsafely(&self, span: Mark) -> &'s [u8] {
        unsafe { self.get_unchecked(span.start..span.end) }
    }
}

impl YamlBuffer for () {
    fn append<'s, S>(&mut self, _src: &'s S, _mark: &mut Mark) {}
}
