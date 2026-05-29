#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use core::fmt::{Debug, Display, Formatter};
use core::i64;
use serde_core::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, StdError};
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
    last_event: YamEvent<'de>,
    has_peeked: bool,
}

impl<'a> YamlIterDeserializer<'a, StrSource<'a>> {
    pub fn new(source: &'a str) -> Self {
        YamlIterDeserializer {
            yaml_iter: ParserIter::new(StrSource::new(source)),
            last_event: YamEvent::DocStart,
            has_peeked: false,
        }
    }
}

impl<'a, R> YamlIterDeserializer<'a, R>
where
    R: Source,
{
    fn next_el(&mut self) -> Option<YamEvent<'a>> {
        if self.has_peeked {
            self.has_peeked = false;
            return Some(self.last_event.clone());
        }

        self.has_peeked = true;
        self.last_event = self.yaml_iter.next()?;
        Some(self.last_event.clone())
    }

    fn skip(&mut self) {
        self.has_peeked = false;
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

macro_rules! parse_from_cow {
    ($e:expr, $v:expr, $method:ident, $t:ty) => {
        match $e.parse::<$t>() {
            Ok(i) => $v.$method(i),
            Err(_) => Err(DeYamlError(YamlError::Custom {
                info: format!("Failed to parse {:?} from scalar value", stringify!($t)),
            })),
        }
    };
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
        match self.skip_doc() {
            Some(YamEvent::MapStart(_, _)) => visitor.visit_map(MapCollection::new(self)),
            Some(YamEvent::SeqStart(_, _)) => visitor.visit_seq(SeqCollection::new(self)),
            Some(YamEvent::Scalar(scalr)) => {
                self.skip();
                self.resolve_scalar(scalr, visitor)
            }
            e => Err(DeYamlError(YamlError::Custom {
                info: format!("Unexpected event (can only process DocStart): {:?}", e),
            })),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.has_peeked = false;
            parse_from_cow!(&scalar.value, visitor, visit_i8, i8)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for i32 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.has_peeked = false;
            parse_from_cow!(&scalar.value, visitor, visit_i16, i16)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for i16 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.skip();
            parse_from_cow!(&scalar.value, visitor, visit_i32, i32)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for i32 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.skip();
            parse_from_cow!(&scalar.value, visitor, visit_i64, i64)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for i64 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.has_peeked = false;
            parse_from_cow!(&scalar.value, visitor, visit_u8, u8)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for u8 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.has_peeked = false;
            parse_from_cow!(&scalar.value, visitor, visit_u16, u16)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for u16 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.skip();
            parse_from_cow!(&scalar.value, visitor, visit_u32, u32)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for u32 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc() {
            self.skip();
            parse_from_cow!(&scalar.value, visitor, visit_u64, u64)
        } else {
            Err(DeYamlError(YamlError::Custom {
                info: "Expected scalar event for u64 deserialization".to_string(),
            }))
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeYamlError(YamlError::Custom {
            info: "Enum deserialization not supported".to_string(),
        }))
    }

    forward_to_deserialize_any! {
        bool i128 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de, R> YamlIterDeserializer<'de, R>
where
    R: Source,
{
    fn skip_doc<'a>(&'a mut self) -> Option<YamEvent<'de>> {
        match self.next_el() {
            Some(YamEvent::DocStart) => {
                self.skip();
                self.next_el()
            }
            ev => ev,
        }
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
                    info: format!("Expected map key, found {:?}", self.iter.last_event),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.iter)?;
        Ok(val)
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
                    info: format!("Expected seq, found {:?}", self.iter.last_event),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }
}

struct Enum<'a, 'de: 'a, R>
where
    R: Source,
{
    de: &'a mut YamlIterDeserializer<'de, R>,
}

impl<'a, 'de, R> Enum<'a, 'de, R>
where
    'de: 'a,
    R: Source,
{
    fn new(de: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        Enum { de }
    }
}
