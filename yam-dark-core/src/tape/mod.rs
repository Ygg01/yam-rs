mod events;

use alloc::string::{String, ToString};
pub use events::EventListener;
use yam_common::ScalarType;

#[allow(dead_code)]
pub enum Node<'input> {
    /// A string, located inside the input slice
    String(&'input str),
    /// A `Map` given the `size` starts here.
    /// The values are keys and value, alternating.
    Map {
        /// Numbers of keys in the map
        len: usize,
        /// Total number of nodes in the map, including sub-elements
        count: usize,
    },
    /// A `Sequence` given size starts here
    Sequence {
        /// The number of elements in the array
        len: usize,
        /// Total number of nodes in the array, including sub-elements.
        count: usize,
    },
    /// A static node that is interned into the tape. It can be directly taken a
    /// and isn't nested.
    Static(StaticNode),
}

#[allow(dead_code)]
pub enum StaticNode {
    /// The null value
    Null,
    /// A boolean value
    Bool(bool),
    /// A floating point value
    F64(f64),
    /// A signed 64-bit integer.
    I64(i64),
}

pub struct StringTape {
    pub buff: String,
}

impl<'a> EventListener<'a> for StringTape {
    type ScalarValue = &'a str;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue, scalar_type: ScalarType) {
        self.buff.push_str(&scalar_type.to_string());
        self.buff.push_str(scalar_value);
    }
}
