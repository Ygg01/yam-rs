use alloc::vec::Vec;
use yam_common::Mark;

/// A trait representing an event listener for scalar values in a processing or parsing context.
///
/// This trait defines callback methods to handle values when they are encountered,
/// allowing implementers to perform specific actions or computations in response to the events.
/// The associated type `Value<'a>` allows flexibility in what type of data is used to represent
/// scalar values, with a lifetime parameter to ensure it can summarize borrowed data if needed.
///
/// # Associated Types
/// * `Value<'a>`: A type representing the scalar value to be processed. Its lifetime supports
/// borrowing behavior and is tied to the source of being parsed.
///
/// # Required Methods
///
/// ## `on_scalar`
///
/// Called when a new scalar value is encountered.
///
/// ### Parameters:
/// - `value`: The scalar value to be handled. The type of this value is defined by the associated type `Value<'a>`.
/// - `scalar_type`: The type of scalar value, typically represented by an external enum like `ScalarType`.
/// - `mark`: Additional metadata or context associated with the scalar value.
///
///
/// ## `on_scalar_continued`
///
/// Called when a scalar value continues, indicating that a composite or streaming scalar sequence
/// provides additional data.
///
/// ### Parameters:
/// - `value`: The additional scalar value to be handled, building on a prior call to `on_scalar`. The type of this value is defined by the associated type `Value<'a>`.
/// - `_scalar_type`: The type of scalar value, typically represented by an external enum like `ScalarType`.
/// - `mark`: Additional metadata or context associated with the continuation of the scalar value.
///
/// # Notes:
/// Implementers of this trait should ensure that the handling of `on_scalar` and `on_scalar_continued`
/// correctly reflects the logic for processing the scalar values in their context, and that they appropriately
/// use the metadata provided by the `mark` parameter.
pub trait EventListener {
    /// The type of scalar value to be handled.
    type Value<'a>;

    /// Event handler called on event start
    fn on_doc_start(&mut self) {
        // Do nothing
    }

    /// Event handler called when a scalar value is first encountered.
    fn on_scalar(&mut self, value: &[u8], mark: Mark);

    /// Event handler called when a scalar value is first encountered.
    fn on_scalar_owned(&mut self, value: Vec<u8>);
}
