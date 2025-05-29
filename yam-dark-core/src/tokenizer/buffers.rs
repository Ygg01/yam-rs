/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer<'b> {
    /// Get the underlying buffer.
    fn append<'src: 'b>(&mut self, src: &'src [u8]) -> &'b [u8];
}

pub trait YamlSource<'s> {
    fn get_span_unsafely(&self, start: usize, end: usize) -> &'s [u8];
}

impl<'b> YamlBuffer<'b> for () {
    fn append<'src: 'b>(&mut self, src: &'src [u8]) -> &'b [u8] {
        src
    }
}

impl<'s> YamlSource<'s> for &'s [u8] {
    fn get_span_unsafely(&self, start: usize, end: usize) -> &'s [u8] {
        unsafe { self.get_unchecked(start..end) }
    }
}
