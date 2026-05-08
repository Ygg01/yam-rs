#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use core::fmt::{Debug, Display, Formatter};
use serde_core::de::StdError;
use serde_core::{Deserializer, de};
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

impl<'de, R, V> Deserializer<'de> for &'de mut YamlIterDeserializer<'de, R, V> {
    type Error = DeYamlError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        for x in self.yaml_iter {
            match x {
                YamEvent::DocStart => {}
                YamEvent::DocEnd => {}
                YamEvent::StreamEnd => {}
                YamEvent::Alias(_) => {}
                YamEvent::Scalar(_) => {}
                YamEvent::SeqStart(_, _) => {}
                YamEvent::SeqEnd => {}
                YamEvent::MapStart(_, _) => {}
                YamEvent::MapEnd => {}
            }
        }
    }
}
