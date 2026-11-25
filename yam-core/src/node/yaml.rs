use crate::prelude::{
    IsEmpty, NodeType, Span, Tag, ToMut, ToMutStr, YamLoader, YamlAccessError, YamlData,
    YamlDocAccess, YamlEntry, YamlError, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

///
/// Basic borrowed YAML data structure.
///
/// The `Yaml` struct is a generic wrapper around `YamlData` with support for specifying
/// a custom floating-point type. By default, the floating-point type is set to `f64`.
///
/// # Type Parameters
/// - `'a`: The lifetime of the YAML data.
/// - `FP`: The floating-point type to be used within the YAML structure. Defaults to `f64`.
///
/// # Fields
/// - `0`: A public field containing the `YamlData` associated with this `Yaml` instance.
///
/// # Derives
/// - `PartialEq`: Enables equality and inequality comparisons for `Yaml`.
/// - `Debug`: Enables formatting the `Yaml` struct for debugging purposes.
///
/// # Examples
/// ```rust
/// use yam_core::node::YamlScalar;
/// use yam_core::prelude::{Yaml, YamlData};
///
/// // Example usage with default floating-point type (f64)
/// let yaml: Yaml = Yaml::from(3.1);
///
/// // Example usage with custom floating-point type
/// let yaml_custom: Yaml<f32> = Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(2.3f32)));
/// ```
#[derive(PartialEq, Debug)]
pub struct Yaml<'a, FP = f64, INT = i64, STR = Cow<'a, str>>(pub YamlData<'a, Self, FP, INT, STR>);

impl<FP, STR, INT> Clone for Yaml<'_, FP, INT, STR>
where
    FP: Copy,
    INT: Copy,
    STR: Clone,
{
    fn clone(&self) -> Self {
        Yaml(self.0.clone())
    }
}

impl<'a, FP, INT, STR> YamlDocAccess<'a> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64>,
    INT: Copy + ToMut<i64>,
    STR: Clone + ToMutStr<'a>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

    fn is_non_empty_collection(&self) -> bool {
        match &self.0 {
            YamlData::Sequence(s) => !s.is_collection_empty(),
            YamlData::Mapping(m) => !m.is_collection_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_mut_val()),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_mut_val()),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match &self.0 {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match &mut self.0 {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match &self.0 {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match &mut self.0 {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_mut_str()),
            _ => None,
        }
    }

    unsafe fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.0 {
            YamlData::Sequence(s) => s,
            _ => unreachable!("YamlData::mapping_mut() on a non-sequence node"),
        }
    }

    unsafe fn sequence(&self) -> &Self::SequenceNode {
        match &self.0 {
            YamlData::Sequence(s) => s,
            _ => unreachable!("YamlData::sequence on a non-sequence node"),
        }
    }

    unsafe fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.0 {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    unsafe fn mapping(&self) -> &Self::MappingNode {
        match &self.0 {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping() called with non-mapping"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.0 {
            YamlData::Tagged(tag, ..) => Some(tag.clone().into_owned()),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        self.0.get_type()
    }

    fn into_string(self) -> Option<String> {
        match self.0 {
            YamlData::Scalar(YamlScalar::String(mut s)) => Some(s.as_mut_str().to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self.0 {
            YamlData::Mapping(s) => Some(s),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self.0 {
            YamlData::Sequence(s) => Some(s),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'a, Tag>) -> Self {
        Yaml(YamlData::Tagged(tag, Box::new(self)))
    }

    fn bad_span_value(_span: Span) -> Self {
        Yaml(YamlData::BadValue)
    }

    fn bad_value() -> Self {
        Yaml(YamlData::BadValue)
    }

    #[inline]
    fn null() -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Null(PhantomData)))
    }

    fn from_bool(b: bool) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Bool(b)))
    }

    fn from_char(chr: char) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_string(chr))))
    }

    fn from_integer(int: impl Into<i64>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(INT::into_owned(
            int.into(),
        ))))
    }

    fn from_string(input: impl Into<String>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_string(
            input,
        ))))
    }

    fn from_str(input: &'a str) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_str(input))))
    }

    fn from_float(input: impl Into<f64>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(FP::into_owned(
            input.into(),
        ))))
    }
}

impl<'a, FP> From<YamlData<'a, Self, FP>> for Yaml<'a, FP> {
    fn from(value: YamlData<'a, Self, FP>) -> Self {
        Yaml(value)
    }
}

impl<'a, FP> From<YamlScalar<'a, FP>> for Yaml<'a, FP> {
    fn from(value: YamlScalar<'a, FP>) -> Self {
        Yaml(YamlData::Scalar(value))
    }
}

impl<'a> Yaml<'a> {
    ///
    /// Loads a vector of YAML document from the given input string.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into a string slice (`&str`)
    ///   using the `AsRef<str>` trait, typically `String` or `&str`.
    ///
    /// # Parameters
    /// - `input`: The YAML input as a string-like object. This is the source from
    ///   which the function attempts to parse a single YAML document.
    ///
    /// # Returns
    /// - `Ok(Node)`: On success, returns the first YAML document (`Node`) parsed from the input string.
    /// - `Err(YamlError)`: Returns an error if:
    ///   - The input is invalid or cannot be parsed due to syntax or structural issues.
    ///   - No YAML document is found within the input (`YamlError::NoDocument`).
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if no valid YAML documents are found in the input.
    /// - Any other error encountered during parsing, as propagated from the parser.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamLoader};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match Yaml::load_from(&yaml_input) {
    ///     Ok(node) => println!("Parsed Node: {node:?}"),
    ///     Err(e) => eprintln!("Error loading YAML: {e}"),
    /// }
    /// ```
    ///
    ///
    pub fn load_from<S: AsRef<str>>(input: S) -> Result<Vec<Yaml<'a>>, YamlError> {
        YamLoader::<Yaml<'a>>::load_from(input)
    }

    ///
    /// Loads a single YAML document from the given input string.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into a string slice (`&str`)
    ///   using the `AsRef<str>` trait, typically `String` or `&str`.
    ///
    /// # Parameters
    /// - `input`: The YAML input as a string-like object. This is the source from
    ///   which the function attempts to parse a single YAML document.
    ///
    /// # Returns
    /// - `Ok(Node)`: On success, returns the first YAML document (`Node`) parsed from the input string.
    /// - `Err(YamlError)`: Returns an error if:
    ///   - The input is invalid or cannot be parsed due to syntax or structural issues.
    ///   - No YAML document is found within the input (`YamlError::NoDocument`).
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if no valid YAML documents are found in the input.
    /// - Any other error encountered during parsing, as propagated from the parser.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamLoader};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match Yaml::load_single(&yaml_input) {
    ///     Ok(node) => println!("Parsed Node: {:?}", node),
    ///     Err(e) => eprintln!("Error loading YAML: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    /// This function expects only a single YAML document in the input. Any additional
    /// documents beyond the first will be ignored.
    ///
    pub fn load_single<S: AsRef<str>>(input: S) -> Result<Yaml<'a>, YamlError> {
        YamLoader::<Yaml<'a>>::load_single(input)
    }
}

impl<'a, FP, INT, STR> Index<usize> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        let typ = self.0.get_type();
        let ind = Yaml::from_integer(i64::try_from(index).expect("Index less than i64::MAX"));
        match typ {
            // SAFETY: can call mapping on a Mapping node.
            NodeType::Mapping => unsafe {
                &self.mapping().iter().find(|x| x.key == ind).unwrap().value
            },
            // SAFETY: can call sequence on a Sequence node.
            NodeType::Sequence => unsafe { self.sequence().index(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, FP, INT, STR> IndexMut<usize> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_integer(i64::try_from(index).expect("Index less than i64::MAX"));
        match typ {
            // SAFETY: can call mapping_mut on a Mapping node.
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .key
            },
            // SAFETY: Can call sequence_mut on a Sequence node.
            NodeType::Sequence => unsafe { self.sequence_mut().index_mut(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, 'k, FP, INT, STR> Index<&'k str> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: &'k str) -> &Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_string(index);
        match typ {
            // SAFETY: Can call mapping on a Mapping node.
            NodeType::Mapping => unsafe {
                &self.mapping().iter().find(|x| x.key == ind).unwrap().value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, 'k, FP, INT, STR> IndexMut<&'k str> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    fn index_mut(&mut self, index: &'k str) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_string(index);
        match typ {
            // SAFETY: Can call mapping_mut on a Mapping node.
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

// Equality comparators

/// Macro for fast implementation generation
/// Ensures all types have easy to access conversion.
///
/// # Params:
/// - `$ty` - Ident of type being compared
/// - `$as_ty` - Ident (name) of a method from `YamDocAccess` that provides access to underlying type
/// - `$from`- Optional. If present the macro will use `$from::from(other)` conversion.
///   otherwise it doesn't use any conversion.
macro_rules! partialeq_numeric {
    ($($as_method:ident $num:ty [$($ty:ty)*])*) => {
        $($(
            impl<'d, F, I, S>  PartialEq<$ty> for Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64>,
                I: Copy + ToMut<i64>,
                S: Clone + ToMutStr<'d>,
            {
                fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }

            impl<'d, F, I, S>  PartialEq<Yaml<'d, F, I, S> > for $ty
            where
                F: Copy + ToMut<f64>,
                I: Copy + ToMut<i64>,
                S: Clone + ToMutStr<'d>,
            {
                fn eq(&self, other: &Yaml<'d, F, I, S>) -> bool {
                    other.$as_method() == Some(*self as $num)
                }
            }

            impl<'d, F, I, S> PartialEq<$ty> for &Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64>,
                I: Copy + ToMut<i64>,
                S: Clone + ToMutStr<'d>,
            {
                fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }

            impl<'a, 'd, F, I, S > PartialEq<$ty> for &mut Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64>,
                I: Copy + ToMut<i64>,
                S: Clone + ToMutStr<'d>,
            {    fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }
        )*)*
    }
}

partialeq_numeric!(
    as_i64 i64 [i8 i16 i32 i64 isize]
    as_i64 i64 [u8 u16 u32]
    as_f64 f64 [f32 f64]
    as_bool bool [bool]
);

/// This conversion is fallible, numbers bigger than i64 can't be stored
/// in `Yaml`
impl<'a, F, I, S> PartialEq<u64> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for u64
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_i64().eq(&i64::try_from(*self).ok())
    }
}

impl<'a, F, I, S> PartialEq<u64> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<u64> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

/// This conversion is potentially fallible on 64-bit systems,
/// numbers bigger than i64 can't be stored
/// in `Yaml`
impl<'a, F, I, S> PartialEq<usize> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for usize
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_i64().eq(&i64::try_from(*self).ok())
    }
}

impl<'a, F, I, S> PartialEq<usize> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<usize> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

/// str conversions are a bit more tricky
impl<'a, F, I, S> PartialEq<&str> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for &str
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(*self))
    }
}

impl<'a, F, I, S> PartialEq<&str> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<&str> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

/// String conversions are a bit more tricky
impl<'a, F, I, S> PartialEq<String> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for String
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(self))
    }
}

impl<'a, F, I, S> PartialEq<String> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<String> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

/// Char conversion is a bit more involved than others.
impl<'a, F, I, S> PartialEq<char> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for char
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(&self.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<char> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<char> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'a>,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

// From conversions

macro_rules! simple_from_conv {
    ($from: ty, $method: ident) => {
        impl<'a> From<$from> for Yaml<'a> {
            #[inline]
            fn from(value: $from) -> Self {
                Yaml::$method(value)
            }
        }
    };
}

simple_from_conv!(char, from_char);
simple_from_conv!(String, from_string);
simple_from_conv!(&str, from_string);
simple_from_conv!(bool, from_bool);
simple_from_conv!(f64, from_float);
simple_from_conv!(f32, from_float);
simple_from_conv!(i8, from_integer);
simple_from_conv!(i16, from_integer);
simple_from_conv!(i32, from_integer);
simple_from_conv!(i64, from_integer);
simple_from_conv!(u32, from_integer);
simple_from_conv!(u16, from_integer);
simple_from_conv!(u8, from_integer);

impl<'a, T> From<Vec<T>> for Yaml<'a>
where
    T: Into<Yaml<'a>>,
{
    fn from(value: Vec<T>) -> Self {
        let sequence = value.into_iter().map(T::into).collect::<Vec<Yaml<'a>>>();
        Yaml(YamlData::Sequence(sequence))
    }
}

impl<'a, K, V> From<Vec<(K, V)>> for Yaml<'a>
where
    K: Into<Yaml<'a>>,
    V: Into<Yaml<'a>>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let sequence = value
            .into_iter()
            .map(|(key, value)| YamlEntry::new(key.into(), value.into()))
            .collect();
        Yaml(YamlData::Mapping(sequence))
    }
}
