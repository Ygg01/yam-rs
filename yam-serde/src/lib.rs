#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use serde_core::de::{MapAccess, SeqAccess, Visitor};
use serde_core::{de, forward_to_deserialize_any};
use yam_core::parsing::{Event, Parser, ScalarValue, Source, StrSource};
use yam_core::prelude::YamlError;

#[derive(Debug)]
pub enum YamSerdeError {
    ParsingError(YamlError),
    Custom(String),
}

impl YamSerdeError {
    pub fn from_str(msg: &str) -> Self {
        YamSerdeError::Custom(msg.to_string())
    }
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

struct Deserializer<'de, R: Source> {
    parser: Parser<'de, R>,
    state: State,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum State {
    #[default]
    StreamStart,
    Document,
    Sequence,
}

#[allow(dead_code)]
impl<'a> Deserializer<'a, StrSource<'a>> {
    pub fn from_str<S: AsRef<str>>(input: &'a S) -> Self {
        Self::new(StrSource::new(input.as_ref()))
    }
}

impl<'de, R> Deserializer<'de, R>
where
    R: Source,
{
    fn new(input: R) -> Self {
        Self {
            parser: Parser::new(input),
            state: State::StreamStart,
        }
    }

    fn next_doc_start(&mut self) -> Result<(), YamSerdeError> {
        if let Ok((ev1, _)) = self.parser.next_event_impl()
            && ev1 == Event::StreamStart
        {
            if let Ok((ev2, _)) = self.parser.next_event_impl()
                && matches!(ev2, Event::DocumentStart(_))
            {
                return Ok(());
            }
        }
        Err(YamSerdeError::Custom("Invalid document start".to_string()))
    }

    fn next_doc_end(&mut self) -> Result<(), YamSerdeError> {
        if let Ok((ev1, _)) = self.parser.next_event_impl()
            && ev1 == Event::DocumentEnd
        {
            return Ok(());
        }
        Err(YamSerdeError::from_str("Invalid document end"))
    }

    fn match_initial<V>(&mut self, visitor: V) -> Result<V::Value, YamSerdeError>
    where
        V: Visitor<'de>,
    {
        self.next_doc_start()?;
        self.state = State::Document;
        let x = self.match_document(visitor)?;
        self.next_doc_end()?;
        Ok(x)
    }

    fn match_document<V>(&mut self, visitor: V) -> Result<V::Value, YamSerdeError>
    where
        V: Visitor<'de>,
    {
        let x = self
            .parser
            .next_event_impl()
            .map_err(|x| YamSerdeError::ParsingError(x))?;
        match x.0 {
            Event::Nothing => visitor.visit_none(),
            Event::StreamStart | Event::DocumentStart(_) => {
                return Err(YamSerdeError::from_str("Unexpected start of document"));
            }
            Event::StreamEnd | Event::DocumentEnd => {
                return Err(YamSerdeError::from_str("Unexpected end of document"));
            }
            Event::Alias(id) => self.resolve_alias(id, visitor),
            Event::Comment(_) => self.match_document(visitor),
            Event::Scalar(x) => self.resolve_scalar(x, visitor),
            Event::SequenceStart(_, _) => {}
            Event::SequenceEnd => {}
            Event::MappingStart(_, _) => {}
            Event::MappingEnd => {}
        }
    }

    fn resolve_alias<V>(&self, id: usize, vistor: V) -> Result<V::Value, YamSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(YamSerdeError::from_str("Alias resolution not implemented"))
    }

    fn resolve_scalar<V>(&self, id: ScalarValue<'de>, vistor: V) -> Result<V::Value, YamSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(YamSerdeError::from_str("Scalar resolution not implemented"))
    }
}

impl<'de, R> de::Deserializer<'de> for &mut Deserializer<'de, R>
where
    R: Source,
{
    type Error = YamSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.state {
            State::StreamStart => self.match_initial(visitor),
            State::Document => self.match_document(visitor),
            _ => Err(YamSerdeError::Custom("Unexpected state".to_string())),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
