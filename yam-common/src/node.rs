use crate::{Mapping, Sequence, Tag, YamlDoc, YamlEntry};
use std::borrow::Cow;

///
/// Represents a cloned YAML node with various possible types of values.
///
/// The `YamlCloneNode` enum provides a structured way to represent YAML values
/// in Rust. Each variant corresponds to a possible type of value that a YAML
/// node can hold, such as `null`, `string`, `integer`, `sequence`, or `mapping`.
///
/// # Type Parameters
///
/// * `'input`: Lifetime parameter for borrowed string slices (`&str`) used in
///   the `String` and `Tagged` variants.
/// * `Node`: Generic parameter for representing nested or child nodes. It must
///   implement the `Clone` trait.
///
/// # Variants
///
/// * `BadValue`:  
///   An invalid value for YAML documents. This variant can represent an
///   uninitialized or erroneous state for a YAML node.
///
/// * `Null`:  
///   Represents a `null` value in the YAML document. This is the default variant.
///
/// * `String(Cow<'input, str>)`:  
///   Represents a YAML string value. The string can either be owned or borrowed,
///   thanks to the use of `Cow`.
///
/// * `Bool(bool)`:  
///   Represents a boolean value (`true` or `false`) in the YAML document.
///
/// * `FloatingPoint(f64)`:  
///   Represents a floating-point number in the YAML document.
///
/// * `Integer(i64)`:  
///   Represents an integer number in the YAML document.
///
/// * `Sequence(Vec<Node>)`:  
///   Represents a sequence (list) of YAML values. It can appear in either
///   flow style (e.g., `[x, x, x]`) or block style:
///   ```yaml
///   - x
///   - x
///   - x
///   ```
///   Each item in the sequence is represented as a `Node`.
///
/// * `Mapping(Vec<YamlEntry<'input, Node>>)`:
///   Represents a mapping (key-value pairs) of YAML values. It can appear in
///   either flow style (e.g., `{x: Y, a: B}`) or block style:
///   ```yaml
///   x: Y
///   a: B
///   ```
///   Each key-value pair is represented as a `YamlEntry`.
///
/// * `Alias(usize)`:  
///   Represents an alias to another YAML node. The alias is denoted by
///   a reference to the index or position of the node it points to.
///
/// * `Tagged(Cow<'input, Tag>, Box<Node>)`:  
///   Represents a tagged YAML value. It includes both a `Tag` (using `Cow`
///   for borrowed or owned strings) and a boxed node as its value.
///
/// # Traits Implementations
///
/// * `Default`: The default value for `YamlCloneNode` is the `Null` variant.
/// * `PartialEq`: Supports equality comparisons between two `YamlCloneNode`
///   instances.
/// * `Clone`: Allows cloning of `YamlCloneNode` instances.
/// * `Debug`: Enables formatted output for debugging purposes.
///
#[derive(Debug, Default, PartialEq, Clone)]
pub enum YamlCloneNode<'input, Node: Clone> {
    /// Invalid value for `YamlDoc`
    BadValue,
    #[default]
    /// Represents a `null` value for `YamlDoc`
    Null,
    /// Represents a YAML string value.
    String(Cow<'input, str>),
    /// Represents a value that's either `true` or `false`
    Bool(bool),
    /// Floating point representation.
    FloatingPoint(f64),
    /// Integer number representation.
    Integer(i64),
    /// Represents a series of values either as flow style like:
    /// ```yaml
    /// [x, x, x]
    /// ```
    /// or block style like
    /// ```yaml
    /// - x
    /// - x
    /// - x
    /// ```
    Sequence(Vec<Node>),

    /// Represents a series of key to map values either in flow style like:
    /// ```yaml
    /// {x: Y, a: B}
    /// ```
    /// or block style like
    /// ```yaml
    /// x: Y
    /// a: B
    /// ```
    Mapping(Vec<YamlEntry<'input, Node>>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged YamlDoc value, contains a [`Tag`] and a node that's a [`Box<Node>`]
    Tagged(Cow<'input, Tag>, Box<Node>),
}

impl<'input, T: Clone> From<YamlDoc<'input>> for YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>>,
{
    fn from(value: YamlDoc<'input>) -> Self {
        match value {
            YamlDoc::BadValue => YamlCloneNode::BadValue,
            YamlDoc::Null => YamlCloneNode::Null,
            YamlDoc::String(x) => YamlCloneNode::String(x),
            YamlDoc::Bool(x) => YamlCloneNode::Bool(x),
            YamlDoc::Alias(x) => YamlCloneNode::Alias(x),
            YamlDoc::FloatingPoint(x) => YamlCloneNode::FloatingPoint(x),
            YamlDoc::Integer(x) => YamlCloneNode::Integer(x),
            YamlDoc::Sequence(s) => YamlCloneNode::from_sequence(s),
            YamlDoc::Mapping(m) => YamlCloneNode::from_mapping(m),
            YamlDoc::Tagged(tag, data) => YamlCloneNode::Tagged(tag, Box::new((*data).into())),
        }
    }
}

impl<'input, T> YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>> + Clone,
{
    fn from_sequence(sequence: Sequence<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Sequence(sequence.into_iter().map(Into::into).collect())
    }

    fn from_mapping(mapping: Mapping<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Mapping(
            mapping
                .into_iter()
                .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                .collect(),
        )
    }
}
