use yam_common::Mark;

/// Trait for buffers used in yam.rs
///
/// It allows abstracting over owned or borrowed buffers, and operations like moving stuff into it.
pub trait YamlBuffer {
    /// Get the underlying buffer.
    fn append<S>(&mut self, src: &S, mark: &mut Mark);
}

impl YamlBuffer for () {
    fn append<'s, S>(&mut self, _src: &S, _mark: &mut Mark) {}
}

/// Traits for input sources used in yam.rs
///
/// It allows abstracting over input strings and buffers
pub trait YamlSource<'s> {
    unsafe fn get_span_unsafely(&self, span: Mark) -> &'s [u8];
    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8;
    fn has_more(&self) -> bool;
}

impl<'s> YamlSource<'s> for &'s str {
    unsafe fn get_span_unsafely(&self, span: Mark) -> &'s [u8] {
        self.get_unchecked(span.start..span.end).as_bytes()
    }

    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8 {
        *self.as_bytes().get_unchecked(pos)
    }

    fn has_more(&self) -> bool {
        false
    }
}

impl<'s> YamlSource<'s> for &'s [u8] {
    unsafe fn get_span_unsafely(&self, span: Mark) -> &'s [u8] {
        self.get_unchecked(span.start..span.end)
    }

    unsafe fn get_u8_unchecked(&self, pos: usize) -> u8 {
        *self.get_unchecked(pos)
    }

    fn has_more(&self) -> bool {
        false
    }
}
