#[allow(dead_code)]
pub enum Node<'input> {
    /// A string, located inside the input slice
    String(&'input str),
    /// A `Map` given the `size` starts here.
    /// The values are keys and value, alternating.
    Map {
        /// Numbers of keys in object
        len: usize,
        /// Total number of nodes in object, including sub-elements
        count: usize,
    },
    /// A `Sequence` given size starts here
    Sequence {
        /// The number of elements in the array
        len: usize,
        /// Total number of nodes in object, including sub-elements.
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
