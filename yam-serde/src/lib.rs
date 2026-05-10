#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use core::fmt::{Debug, Display, Formatter};
use serde_core::de::StdError;
use serde_core::{Deserializer, de, forward_to_deserialize_any};
use yam_core::LazyExpander;
use yam_core::node::YamlScalar;
use yam_core::parsing::parser_iter::YamEvent;
use yam_core::parsing::{ParserIter, ScalarValue, StrSource};
use yam_core::prelude::{Source, YamlError};

struct YamlIterDeserializer<'de, R>
where
    R: Source,
    R: 'de,
{
    yaml_iter: ParserIter<'de, R>,
    alias: BTreeMap<usize, LazyExpander<'de>>,
}

impl<'a> YamlIterDeserializer<'a, StrSource<'a>> {
    pub fn new(source: &'a str) -> Self {
        YamlIterDeserializer {
            yaml_iter: ParserIter::new(StrSource::new(source)),
            alias: BTreeMap::new(),
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
struct DeYamlError(YamlError);

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

impl<'de, R> YamlIterDeserializer<'de, R>
where
    R: Source,
{
    fn resolve_scalar<V: de::Visitor<'de>>(
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
            Some(YamlScalar::Null(_)) => visitor.visit_none(),
            None => Err(DeYamlError(YamlError::new_custom("Failed to parse scalar"))),
        }
    }
}

impl<'de, 'a, R> Deserializer<'de> for &'a mut YamlIterDeserializer<'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.yaml_iter.next() {
            Some(YamEvent::Scalar(scalar)) => self.resolve_scalar(scalar, visitor),
            Some(ev) => {
                let info = format!("Didn't expect to see: {:?}", ev);
                Err(DeYamlError(YamlError::Custom { info }))
            }
            None => Err(DeYamlError(YamlError::new_custom("Unexpected end"))),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
