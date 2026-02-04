#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;
use serde_core::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_core::{de, forward_to_deserialize_any};
use yam_common::loader::LoadableYamlNode;
use yam_common::{Mapping, Sequence, YamlDoc, YamlError};
use yam_core::Source;
use yam_core::saphyr_tokenizer::StrSource;
use yam_core::treebuild::YamlLoader;

pub fn from_str<'a, T>(input: &'a str) -> Result<T, YamSerdeError>
where
    T: de::Deserialize<'a>,
{
    let de = Deserializer::new(StrSource::new(input));
    let value = de::Deserialize::deserialize(de)?;

    Ok(value)
}

pub struct Deserializer<'a, R: Source> {
    input: R,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> Deserializer<'a, StrSource<'a>> {
    pub fn from_str<S: AsRef<str>>(input: &'a S) -> Self {
        Self::new(StrSource::new(input.as_ref()))
    }
}

impl<'a, R> Deserializer<'a, R>
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
            YamSerdeError::Custom(x) => write!(f, "Custom error: {}", x)?,
            YamSerdeError::ParsingError(yaml_error) => write!(f, "Parsing error: {}", yaml_error)?,
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
        let doc = match YamlLoader::<YamlDoc<'de>>::load_single_from_parser(self.input) {
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
    sequence: Sequence<'de>,
    idx: usize,
}

impl<'de> YamSequenceDeserializer<'de> {
    fn new(sequence: Sequence<'de>) -> Self {
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
    mapping: Mapping<'de>,
    idx: usize,
}

impl<'de> YamMapDeserializer<'de> {
    fn new(mapping: Mapping<'de>) -> Self {
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
