#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter};

use serde_core::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, StdError, VariantAccess,
};
use serde_core::{de, forward_to_deserialize_any};
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

    fn peek_null(&mut self) -> bool {
        if let Some(YamEvent::Scalar(scalar)) = self.skip_doc()
            && (scalar.value == "null" || scalar.value == "~")
        {
            true
        } else {
            false
        }
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
            None => Err(DeYamlError::Custom("Failed to parse scalar".to_string())),
        }
    }
}

macro_rules! parse_from_cow {
    ($e:expr, $v:expr, $method:ident, $t:ty) => {
        match $e.parse::<$t>() {
            Ok(i) => $v.$method(i),
            Err(_) => Err(DeYamlError::Custom(format!(
                "Failed to parse {:?} from scalar value",
                stringify!($t)
            ))),
        }
    };
}

impl<'a, 'de, R> de::Deserializer<'de> for &'a mut YamlIterDeserializer<'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.skip_doc() {
            Some(YamEvent::MapStart(_, _)) => self.deserialize_map(visitor),
            Some(YamEvent::SeqStart(_, _)) => self.deserialize_seq(visitor),
            Some(YamEvent::Scalar(scalr)) => {
                self.skip();
                self.resolve_scalar(scalr, visitor)
            }
            e => Err(DeYamlError::Custom(format!(
                "Unexpected event (can only process DocStart): {:?}",
                e
            ))),
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
            Err(DeYamlError::Custom(
                "Expected scalar event for i32 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for i16 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for i32 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for i64 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for i8 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for u16 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for u32 deserialization".to_string(),
            ))
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
            Err(DeYamlError::Custom(
                "Expected scalar event for u64 deserialization".to_string(),
            ))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.peek_null() {
            self.skip();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.peek_null() {
            self.skip();
            visitor.visit_unit()
        } else {
            Err(DeYamlError::ExpectedNull)
        }
    }

    forward_to_deserialize_any! {
        bool i128 u128 f32 f64 char str string
        bytes byte_buf unit_struct newtype_struct tuple
        tuple_struct struct ignored_any
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if !matches!(self.skip_doc(), Some(YamEvent::SeqStart(_, _))) {
            return Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                expected: "SeqStart",
                found: self.last_event.as_simple_str(),
            }));
        }
        self.skip();
        let val = visitor.visit_seq(SeqCollection::new_seq(self))?;
        if !matches!(self.last_event, YamEvent::SeqEnd)
            && !matches!(self.next_el(), Some(YamEvent::SeqEnd))
        {
            return Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                expected: "SeqEnd",
                found: self.last_event.as_simple_str(),
            }));
        }
        self.skip();
        Ok(val)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if !matches!(self.skip_doc(), Some(YamEvent::MapStart(_, _))) {
            return Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                expected: "MapStart",
                found: self.last_event.as_simple_str(),
            }));
        }
        self.skip();
        let val = visitor.visit_map(SeqCollection::new_map(self))?;
        if !matches!(self.last_event, YamEvent::MapEnd)
            && !matches!(self.next_el(), Some(YamEvent::MapEnd))
        {
            return Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                expected: "MapEnd",
                found: self.last_event.as_simple_str(),
            }));
        }
        self.skip();
        Ok(val)
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
        match self.skip_doc() {
            Some(YamEvent::Scalar(ScalarValue { value, .. })) => {
                self.skip();
                visitor.visit_enum(value.into_deserializer())
            }
            Some(YamEvent::MapStart(_, _)) => {
                self.skip();
                let value = visitor.visit_enum(Enum::new(self))?;

                if !matches!(self.next_el(), Some(YamEvent::MapEnd)) {
                    return Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                        found: self.last_event.as_simple_str(),
                        expected: "MapEnd",
                    }));
                }
                Ok(value)
            }
            _ => Err(DeYamlError::Custom("Expected enum".to_string())),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
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
pub enum DeYamlError {
    ParserError(YamlError),
    Custom(String),
    ExpectedStringInNewType,
    ExpectedNull,
}

impl StdError for DeYamlError {}

impl Display for DeYamlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DeYamlError::ParserError(err) => write!(f, "ParserError: {}", err)?,
            DeYamlError::Custom(msg) => write!(f, "Custom: {}", msg)?,
            DeYamlError::ExpectedStringInNewType => write!(f, "Expected String:")?,
            DeYamlError::ExpectedNull => write!(f, "Expected Null")?,
        };
        Ok(())
    }
}

impl de::Error for DeYamlError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        let info = format!("{}", msg);
        DeYamlError::Custom(info)
    }
}

struct SeqCollection<'a, 'de: 'a, R>
where
    R: Source,
{
    iter: &'a mut YamlIterDeserializer<'de, R>,
    map: bool,
}

impl<'a, 'de, R> SeqCollection<'a, 'de, R>
where
    R: Source,
{
    fn new_seq(iter: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        SeqCollection { iter, map: false }
    }

    fn new_map(iter: &'a mut YamlIterDeserializer<'de, R>) -> Self {
        SeqCollection { iter, map: true }
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
                Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                    expected: "SeqEnd",
                    found: self.iter.last_event.as_simple_str(),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }
}

impl<'de, 'a, R> MapAccess<'de> for SeqCollection<'a, 'de, R>
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
                Err(DeYamlError::ParserError(YamlError::UnExpectedEvent {
                    expected: "MapEnd",
                    found: self.iter.last_event.as_simple_str(),
                }))
            }
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // TODO
        let val = seed.deserialize(&mut *self.iter)?;
        Ok(val)
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

impl<'a, 'de, R> EnumAccess<'de> for Enum<'a, 'de, R>
where
    'de: 'a,
    R: Source,
{
    type Error = DeYamlError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'a, 'de, R> VariantAccess<'de> for Enum<'a, 'de, R>
where
    'de: 'a,
    R: Source,
{
    type Error = DeYamlError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        todo!()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut *self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut *self.de, visitor)
    }
}
