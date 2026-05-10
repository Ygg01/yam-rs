#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use core::fmt::{Debug, Display, Formatter};
use serde_core::de::StdError;
use serde_core::{Deserializer, de, forward_to_deserialize_any};
use yam_core::LazyExpander;
use yam_core::parsing::ParserIter;
use yam_core::parsing::parser_iter::YamEvent;
use yam_core::prelude::{Source, YamlError};

struct YamlIterDeserializer<'de, R>
where
    R: Source,
{
    yaml_iter: ParserIter<'de, R>,
    alias: BTreeMap<usize, LazyExpander<'de>>,
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
    fn resolve_scalar<V: de::Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, DeYamlError> {
    }
}

impl<'de, R> Deserializer<'de> for &'de mut YamlIterDeserializer<'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        for x in self.yaml_iter {
            let info = format!("Didn't expect to see: {:?}", x);
            match x {
                YamEvent::DocStart => {}
                YamEvent::Scalar(scalar) => {}
                YamEvent::SeqStart(_, _) => {}
                YamEvent::MapStart(_, _) => {}
                _ => Err(DeYamlError(YamlError::Custom { info })),
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
