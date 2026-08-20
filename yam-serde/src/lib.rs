#![no_std]

extern crate alloc;
pub mod binary;
pub mod de;
mod escape_str;
#[allow(dead_code, unused_variables)]
pub mod ser;

use crate::de::DeYamlError;
use crate::ser::{FlowStyle, YamlStyle};
use alloc::borrow::Cow;
use alloc::string::String;
use core::fmt::Error;
use yam_core::prelude::ScalarType;

/// Attempts to deserialize an instance of type `T` from a YAML input string.
///
///
/// # Type Parameters
///
/// * `T`: The type you want to deserialize the input string into.
///   It must implement the `serde_core::de::Deserialize` trait.
///
/// # Parameters
///
/// * `input`: A string slice containing the YAML input to be deserialized.
///
/// # Returns
///
/// A `Result` containing either:
/// - The successfully deserialized value of type `T`, or
/// - A `DeYamlError` error if deserialization fails.
///
/// # Errors
///
/// This function returns a `DeYamlError` if any errors occur during the deserialization process,
/// such as invalid YAML syntax or mismatched data types.
///
/// # Examples
///
/// ```
/// use yam_serde::de::DeYamlError;
/// use yam_serde::from_str;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct Config {
///     field: String,
///     value: i32,
/// }
///
/// let yaml_input = r#"
/// field: "example"
/// value: 42
/// "#;
///
/// let result: Result<Config, DeYamlError> = from_str(yaml_input);
/// match result {
///     Ok(config) => {
///         assert_eq!(config.field, "example");
///         assert_eq!(config.value, 42);
///         println!("Successfully deserialized: {:?}", config);
///     },
///     Err(e) => println!("Failed to deserialize: {}", e),
/// }
/// ```
pub fn from_str<'a, T>(input: &'a str) -> Result<T, DeYamlError>
where
    T: serde_core::de::Deserialize<'a>,
{
    let mut de = crate::de::YamIterDeserializer::new(input);
    let value = T::deserialize(&mut de)?;

    Ok(value)
}

/// Converts a serializable value into a  YAML string.
///
/// # Type Parameters
/// - `T`: The type of the value to be serialized, which must implement the `serde_core::ser::Serialize` trait.
///
/// # Parameters
/// - `value`: A reference to the value to serialize.
/// - `formatter`: A `PrettyFormatterConfig` instance that specifies the YAML formatting options.
///
/// # Returns
/// - On success, returns `Ok(String)` containing the serialized and formatted YAML string.
/// - On failure, returns an `Err(Error)` detailing the reason for the serialization failure.
///
/// # Errors
/// This function returns an `Error` if:
/// - Serialization of the input value fails.
/// - The writer fails to finalize the output.
///
/// # Example
/// ```
/// use yam_serde::{to_pretty_string, PrettyFormatterConfig};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person {
///     name: String::from("Alice"),
///     age: 30,
/// };
///
/// let formatter = PrettyFormatterConfig::default();
/// let yaml_string = to_pretty_string(&person, formatter).unwrap();
/// println!("{}", yaml_string);
/// ```
pub fn to_pretty_string<T>(value: &T, formatter: PrettyFormatterConfig) -> Result<String, Error>
where
    T: serde_core::ser::Serialize,
{
    let mut serializer = ser::YamSerializer::new_pretty(String::new(), formatter);
    value.serialize(&mut serializer)?;
    Ok(serializer.writer)
}

#[derive(Debug, Copy, Clone, Default)]
pub enum NullFormat {
    #[default]
    /// Null that has corresponds to JSON null
    /// ```yaml
    /// example: null
    /// ```
    JsonNull,
    /// Null that has a schema built in.
    /// ```yaml
    /// example: !!null null
    /// ```
    TaggedYaml,
    /// Null that's just an empty YAML.
    /// ```yaml
    /// example: # the value of null key is null.
    /// ```
    Plain,
    /// Null used in Yaml 1.1 i.e.
    /// ```yaml
    /// example: ~
    /// ```
    OldYaml,
}

impl NullFormat {
    pub fn to_null_string(self) -> Cow<'static, str> {
        match self {
            NullFormat::JsonNull => Cow::Borrowed("null"),
            NullFormat::TaggedYaml => Cow::Borrowed("!!null null"),
            NullFormat::Plain => Cow::Borrowed(""),
            NullFormat::OldYaml => Cow::Borrowed("~"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PrettyFormatterConfig {
    /// Limit depth
    pub block_depth_limit: u32,

    /// Preferred string length
    pub pref_string_length: u32,

    /// Indentation string
    pub indentor: Cow<'static, str>,

    /// New line string
    pub new_line: Cow<'static, str>,

    /// How to format null
    null_format: Cow<'static, str>,

    /// How to format a string in block style
    pub block_preferred_style: ScalarType,

    /// How to format a string in a root
    pub root_preferred_style: ScalarType,

    /// How to format a string in block style
    pub flow_string_style: FlowStyle,

    pub key_preferred_style: ScalarType,

    /// Whether to prefer string to fit in a single line
    pub compat_strings: bool,

    /// What is the default style for the Root element
    pub root_style: YamlStyle,
}

impl Default for PrettyFormatterConfig {
    fn default() -> Self {
        Self {
            block_depth_limit: 0,
            pref_string_length: 80,
            indentor: Cow::Borrowed("  "),
            new_line: Cow::Borrowed("\n"),
            null_format: Cow::Borrowed(""),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compat_strings: false,
            root_style: YamlStyle::default(),
        }
    }
}

impl PrettyFormatterConfig {
    #[must_use]
    pub fn pretty() -> Self {
        Self {
            block_depth_limit: 10,
            pref_string_length: 80,
            indentor: Cow::Borrowed("  "),
            new_line: Cow::Borrowed("\n"),
            null_format: Cow::Borrowed("null"),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compat_strings: false,
            root_style: YamlStyle::default(),
        }
    }

    #[inline]
    pub fn set_null_format(&mut self, fmt: NullFormat) {
        self.null_format = fmt.to_null_string();
    }
}
