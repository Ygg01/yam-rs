mod eq;
mod from;
mod yam_access;

use crate::prelude::{YamLoader, YamlData, YamlError};
use alloc::borrow::Cow;
use alloc::vec::Vec;

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
