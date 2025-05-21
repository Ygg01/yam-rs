use yam_common::ScalarType;

/// Trait for listening to YAML events and modifying the iterator in place
///
/// # Safety
///
/// 1. Trait assumes UTF8 encoding
/// 2. It's expected that for a given input source, when events containing `start`, `end` offsets
///    that for slicing input as `input[start..end]` will produce a valid UTF8 string.
pub(crate) unsafe trait YamlEventsListener {
    fn on_scalar(&mut self, start: usize, end: usize, scalar_type: ScalarType);
}
