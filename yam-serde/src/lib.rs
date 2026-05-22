#![no_std]

extern crate alloc;

use alloc::format;

use core::fmt::{Debug, Display, Formatter};
use serde_core::de::{DeserializeSeed, MapAccess, SeqAccess, StdError};
use serde_core::{Deserializer, de, forward_to_deserialize_any};
use yam_core::node::YamlScalar;
use yam_core::parsing::parser_iter::YamEvent;
use yam_core::parsing::{ParserIter, ScalarValue, StrSource};
use yam_core::prelude::{Source, YamlError};

struct YamlIterDeserializer<'de, R>
where
    R: Source,
{
    yaml_iter: ParserIter<'de, R>,
    peek_event: YamEvent<'de>,
    is_peeked: bool,
}

impl<'a> YamlIterDeserializer<'a, StrSource<'a>> {
    pub fn new(source: &'a str) -> Self {
        YamlIterDeserializer {
            yaml_iter: ParserIter::new(StrSource::new(source)),
            peek_event: YamEvent::DocStart,
            is_peeked: false,
        }
    }
}

impl<'a, R> YamlIterDeserializer<'a, R>
where
    R: Source,
{
    fn next_el(&mut self) -> Option<YamEvent<'a>> {
        if self.is_peeked {
            self.is_peeked = false;
            return Some(self.peek_event.clone());
        }

        self.is_peeked = true;
        self.peek_event = self.yaml_iter.next()?;
        Some(self.peek_event.clone())
    }

    fn resolve_scalar<V: de::Visitor<'a>>(
        &mut self,
        scalar_value: ScalarValue,
        visitor: V,
    ) -> Result<V::Value, DeYamlError> {
        let scalar = YamlScalar::parse_from_scalar(scalar_value);
        match scalar {
            Some(YamlScalar::Integer(x)) => visitor.visit_i64(x),
            Some(YamlScalar::FloatingPoint(x)) => visitor.visit_f64(x),
            Some(YamlScalar::Bool(x)) => visitor.visit_bool(x),
            Some(YamlScalar::String(x)) => visitor.visit_str(&x),
            Some(YamlScalar::Null(_)) => visitor.visit_unit(),
            None => Err(DeYamlError(YamlError::new_custom("Failed to parse scalar"))),
        }
    }
}

impl<'a, 'de, R> Deserializer<'de> for &'a mut YamlIterDeserializer<'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.next_el() {
            Some(YamEvent::DocStart) => {
                self.is_peeked = false;
                DocSerializer::new(self).deserialize_any(visitor)
            }
            e => Err(DeYamlError(YamlError::Custom {
                info: format!("Unexpected event (can only process DocStart): {:?}", e),
            })),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub fn from_str<'a, T>(input: &'a str) -> Result<T, DeYamlError>
where
    T: de::Deserialize<'a>,
{
    let mut de = YamlIterDeserializer::new(input);
    let value = T::deserialize(&mut de)?;

    Ok(value)
}

#[derive(Debug)]
pub struct DeYamlError(YamlError);

impl StdError for DeYamlError {}

impl Display for DeYamlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DeYamlError").field(&self.0).finish()
    }
}

impl de::Error for DeYamlError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        let info = format!("{}", msg);
        DeYamlError(YamlError::Custom { info })
    }
}

struct DocSerializer<'a, 'de, R>
where
    R: Source,
    'de: 'a,
{
    deserializer: &'a mut YamlIterDeserializer<'de, R>,
}

impl<'a, 'de, R> DocSerializer<'a, 'de, R>
where
    R: Source,
    'de: 'a,
{
    fn new(iter: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        DocSerializer { deserializer: iter }
    }

    fn resolve_scalar<V: de::Visitor<'de>>(
        &mut self,
        scalar_value: ScalarValue,
        visitor: V,
    ) -> Result<V::Value, DeYamlError> {
        self.deserializer.resolve_scalar(scalar_value, visitor)
    }
}

impl<'a, 'de, R> DocSerializer<'a, 'de, R>
where
    R: Source,
    'de: 'a,
{
    fn serialize_any<V>(
        &'a mut self,
        visitor: V,
        event: YamEvent<'de>,
    ) -> Result<V::Value, DeYamlError>
    where
        V: de::Visitor<'de>,
    {
        match event {
            YamEvent::Scalar(scalar_value) => self.resolve_scalar(scalar_value, visitor),
            YamEvent::MapStart(_, _) => self.deserialize_map(visitor),
            YamEvent::SeqStart(_, _) => self.deserialize_seq(visitor),
            e => Err(DeYamlError(YamlError::Custom {
                info: format!("Unexpected event in serialize: {:?}", e),
            })),
        }
    }
}

impl<'a, 'de, R> Deserializer<'de> for &'a mut DocSerializer<'a, 'de, R>
where
    R: Source,
    'de: 'a,
{
    type Error = DeYamlError;
    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.deserializer.next_el() {
            Some(ev) => self.serialize_any(visitor, ev),
            e => Err(DeYamlError(YamlError::Custom {
                info: format!("Unexpected event: {:?}", e),
            })),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if !matches!(self.deserializer.peek_event, YamEvent::SeqStart(_, _)) {
            return Err(DeYamlError(YamlError::Custom {
                info: format!(
                    "Expected SeqStart event, found {:?}",
                    self.deserializer.peek_event
                ),
            }));
        }
        let value = visitor.visit_seq(SeqCollection::new(self.deserializer))?;
        match self.deserializer.peek_event {
            YamEvent::SeqEnd => Ok(value),
            _ => Err(DeYamlError(YamlError::Custom {
                info: format!(
                    "Expected SeqEnd event, found {:?}",
                    self.deserializer.peek_event
                ),
            })),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if !matches!(self.deserializer.peek_event, YamEvent::MapStart(_, _)) {
            return Err(DeYamlError(YamlError::Custom {
                info: format!(
                    "Expected MapStart event, found {:?}",
                    self.deserializer.peek_event
                ),
            }));
        }
        let value = visitor.visit_map(MapCollection::new(self.deserializer))?;
        match self.deserializer.peek_event {
            YamEvent::MapEnd => Ok(value),
            _ => Err(DeYamlError(YamlError::Custom {
                info: format!(
                    "Expected MapEnd event, found {:?}",
                    self.deserializer.peek_event
                ),
            })),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct  tuple
        tuple_struct struct enum identifier ignored_any
    }
}

struct MapCollection<'a, 'de: 'a, R>
where
    R: Source,
{
    iter: &'a mut YamlIterDeserializer<'de, R>,
}

impl<'a, 'de, R> MapCollection<'a, 'de, R>
where
    R: Source,
{
    fn new(iter: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        MapCollection { iter }
    }
}

impl<'de, 'a, R> MapAccess<'de> for MapCollection<'a, 'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next_el() {
            Some(YamEvent::MapEnd) => Ok(None),
            Some(YamEvent::DocEnd | YamEvent::StreamEnd) | None => {
                Err(DeYamlError(YamlError::Custom {
                    info: format!("Expected map key, found {:?}", self.iter.peek_event),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.iter)
    }
}

struct SeqCollection<'a, 'de: 'a, R>
where
    R: Source,
{
    iter: &'a mut YamlIterDeserializer<'de, R>,
}

impl<'a, 'de, R> SeqCollection<'a, 'de, R>
where
    R: Source,
{
    fn new(iter: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        SeqCollection { iter }
    }
}

impl<'de, 'a, R> SeqAccess<'de> for SeqCollection<'a, 'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next_el() {
            Some(YamEvent::SeqEnd) => Ok(None),
            Some(YamEvent::DocEnd | YamEvent::StreamEnd) | None => {
                Err(DeYamlError(YamlError::Custom {
                    info: format!("Expected seq, found {:?}", self.iter.peek_event),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }
}
