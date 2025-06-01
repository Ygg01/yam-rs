use crate::tape::Mark;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer {
    /// Get the underlying buffer.
    fn append(&mut self, src: &[u8]) -> Mark;
}

pub trait YamlSource<'s> {
    fn get_span_unsafely(&self, span: Mark) -> &'s [u8];
}

impl<'s> YamlSource<'s> for &'s [u8] {
    fn get_span_unsafely(&self, span: Mark) -> &'s [u8] {
        unsafe { self.get_unchecked(span.start..span.end) }
    }
}

pub struct DummyBuffer(usize);

impl YamlBuffer for DummyBuffer {
    fn append(&mut self, src: &[u8]) -> Mark {
        let mark = Mark::new(self.0, self.0 + src.len());
        self.0 += src.len();
        mark
    }
}
