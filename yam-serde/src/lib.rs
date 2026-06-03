#![no_std]

extern crate alloc;
pub mod de;
pub mod ser;

use crate::de::DeYamlError;
use crate::ser::SerYamlError;
use alloc::string::String;

/// Attempts to deserialize a YAML input string into a value of type `T`.
///
/// This function leverages the `Deserialize` trait from the `serde` library
/// to convert a YAML string slice into the corresponding Rust data structure.
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
///     Ok(config) => println!("Successfully deserialized: {:?}", config),
///     Err(e) => eprintln!("Failed to deserialize: {}", e),
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

pub fn to_string<T>(value: &T) -> Result<String, SerYamlError>
where
    T: serde_core::ser::Serialize,
{
    let mut serializer = crate::ser::YamSerializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}
