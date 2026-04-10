#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;
use serde_core::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_core::{de, forward_to_deserialize_any};
use yam_core::parsing::{Source, StrSource};
use yam_core::prelude::{YamlDoc, YamlEntry, YamlError};
use yam_core::{YamlDocAccess, YamlLoader};

///
/// Attempts to deserialize a value of type `T` from a given YAML string slice.
///
/// This function takes a string slice as input and attempts to deserialize it
/// into the specified type `T` using the `Deserialize` trait from the `serde`
/// library. The deserialization process may fail if the input string is not
/// valid YAML or if it does not conform to the structure of type `T`.
///
/// # Type Parameters
///
/// * `T`: The type into which the given YAML string will be deserialized.
///   `T` must implement the `Deserialize<'a>` trait.
///
/// # Parameters
///
/// * `input`: A string slice containing the YAML-encoded data to deserialize.
///
/// # Returns
///
/// * `Ok(T)`: Returns an object of type `T` if the deserialization is successful.
/// * `Err(YamSerdeError)`: Returns a `YamSerdeError` if the deserialization fails.
///
/// # Errors
///
/// This function will return a `YamSerdeError` if:
/// * The input string is not valid YAML.
/// * The structure or data in the YAML string does not match the structure of type `T`.
///
/// # Example
///
/// ```rust
/// use yam_serde::{from_str, YamSerdeError};
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct Config {
///     host: String,
///     port: u16,
/// }
///
/// fn main() -> Result<(), YamSerdeError> {
///     let yaml_data = r#"
///         host: localhost
///         port: 8080
///     "#;
///
///     let config: Config = from_str(yaml_data)?;
///     println!("Parsed config: {:?}", config);
///     Ok(())
/// }
/// ```
///
/// In this example, the `from_str` function is used to parse a YAML string into
/// a `Config` struct. If the parsing is successful, the function returns the
/// struct; otherwise, it returns an error.
///
/// # Note
///
/// This function uses `Deserializer` and `StrSource` internally for processing
/// the YAML input.
///
pub fn from_str<'a, T>(input: &'a str) -> Result<T, YamSerdeError>
where
    T: de::Deserialize<'a>,
{
    let de = Deserializer::new(StrSource::new(input));
    let value = de::Deserialize::deserialize(de)?;

    Ok(value)
}

struct Deserializer<'a, R: Source> {
    input: R,
    phantom_data: PhantomData<&'a ()>,
}

#[allow(dead_code)]
impl<'a> Deserializer<'a, StrSource<'a>> {
    pub fn from_str<S: AsRef<str>>(input: &'a S) -> Self {
        Self::new(StrSource::new(input.as_ref()))
    }
}

impl<R> Deserializer<'_, R>
where
    R: Source,
{
    fn new(input: R) -> Self {
        Self {
            input,
            phantom_data: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum YamSerdeError {
    ParsingError(YamlError),
    Custom(String),
}

impl Display for YamSerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            YamSerdeError::Custom(x) => write!(f, "Custom error: {x}")?,
            YamSerdeError::ParsingError(yaml_error) => write!(f, "Parsing error: {yaml_error}")?,
        }
        Ok(())
    }
}

impl de::StdError for YamSerdeError {}

impl de::Error for YamSerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        YamSerdeError::Custom(msg.to_string())
    }
}

impl<'de, R> de::Deserializer<'de> for Deserializer<'de, R>
where
    R: Source,
{
    type Error = YamSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let doc = match YamlLoader::<YamlDoc<'de>>::load_single_source(self.input) {
            Ok(x) => x,
            Err(e) => return Err(YamSerdeError::ParsingError(e)),
        };
        YamDocDeserializer { doc }.deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct YamSequenceDeserializer<'de> {
    sequence: Vec<YamlDoc<'de>>,
    idx: usize,
}

impl<'de> YamSequenceDeserializer<'de> {
    fn new(sequence: Vec<YamlDoc<'de>>) -> Self {
        Self { sequence, idx: 0 }
    }
}

impl<'de> SeqAccess<'de> for YamSequenceDeserializer<'de> {
    type Error = YamSerdeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let element = self.sequence.get_mut(self.idx);
        match element {
            Some(d) => {
                self.idx += 1;
                seed.deserialize(YamDocDeserializer::take_from_ref(d))
                    .map(Some)
            }
            None => Ok(None),
        }
    }
}

struct YamMapDeserializer<'de> {
    mapping: Vec<YamlEntry<'de, YamlDoc<'de>>>,
    idx: usize,
}

impl<'de> YamMapDeserializer<'de> {
    fn new(mapping: Vec<YamlEntry<'de, YamlDoc<'de>>>) -> Self {
        Self { mapping, idx: 0 }
    }
}

impl<'de> MapAccess<'de> for YamMapDeserializer<'de> {
    type Error = YamSerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let entry = self.mapping.get_mut(self.idx);
        match entry {
            Some(entry) => seed
                .deserialize(YamDocDeserializer::take_from_ref(&mut entry.key))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let entry = self.mapping.get_mut(self.idx);
        match entry {
            Some(entry) => {
                // TODO Do we get key or map first??
                self.idx += 1;
                seed.deserialize(YamDocDeserializer::take_from_ref(&mut entry.value))
            }
            // TODO is this safe?
            None => unreachable!("Should not be possible to get here because we had a key"),
        }
    }
}

struct YamDocDeserializer<'de> {
    doc: YamlDoc<'de>,
}

impl<'de> YamDocDeserializer<'de> {
    fn take_from_ref<'a>(new: &'a mut YamlDoc<'de>) -> YamDocDeserializer<'de> {
        Self { doc: new.take() }
    }
}

impl<'de> de::Deserializer<'de> for YamDocDeserializer<'de> {
    type Error = YamSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.doc {
            YamlDoc::BadValue => Err(YamSerdeError::Custom(
                "Bad value during parsing found!".to_string(),
            )),
            YamlDoc::Null => visitor.visit_none(),
            YamlDoc::Bool(b) => visitor.visit_bool(b),
            YamlDoc::String(s) => visitor.visit_str(&s),
            YamlDoc::FloatingPoint(s) => visitor.visit_f64(s),
            YamlDoc::Integer(s) => visitor.visit_i64(s),
            YamlDoc::Sequence(seq) => visitor.visit_seq(YamSequenceDeserializer::new(seq)),
            YamlDoc::Mapping(map) => visitor.visit_map(YamMapDeserializer::new(map)),
            YamlDoc::Alias(_) | YamlDoc::Tagged(..) => unimplemented!("TODO"),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
