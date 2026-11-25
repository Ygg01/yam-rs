use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use serde_core::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, StdError, VariantAccess,
    Visitor,
};
use serde_core::{de, forward_to_deserialize_any};

use crate::binary;
use crate::binary::DecodeError;
use crate::parsing::parser_iter::YamEvent;
use crate::parsing::{InMemSource, ScalarValue, YamParserIter};
use crate::prelude::{Source, YamlScalar};

pub(crate) struct YamIterDeserializer<'de, R>
where
    R: Source,
{
    yaml_iter: YamParserIter<'de, R>,
    last_event: YamEvent<'de>,
    has_peeked: bool,
}

impl<'a> YamIterDeserializer<'a, InMemSource<'a>> {
    pub fn new(source: &'a str) -> Self {
        YamIterDeserializer {
            yaml_iter: YamParserIter::new(InMemSource::new(source)),
            last_event: YamEvent::DocStart,
            has_peeked: false,
        }
    }
}

impl<'de, R> YamIterDeserializer<'de, R>
where
    R: Source,
{
    fn next_el(&mut self) -> Option<YamEvent<'de>> {
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
        match self.skip_doc() {
            Some(YamEvent::Scalar(scalar)) => scalar.is_null(),
            None => true,
            _ => false,
        }
    }

    fn resolve_scalar<V: Visitor<'de>>(
        scalar_value: ScalarValue<'de>,
        visitor: V,
    ) -> Result<V::Value, DeYamlError> {
        let scalar = YamlScalar::parse_from_scalar(scalar_value);
        match scalar {
            Some(YamlScalar::Integer(int)) => visitor.visit_i64(int),
            Some(YamlScalar::FloatingPoint(float)) => visitor.visit_f64(float),
            Some(YamlScalar::Bool(boolean)) => visitor.visit_bool(boolean),
            Some(YamlScalar::String(cow_str)) => match cow_str {
                Cow::Borrowed(input) => visitor.visit_borrowed_str(input),
                Cow::Owned(input) => visitor.visit_str(&input),
            },
            Some(YamlScalar::Null(_)) => visitor.visit_unit(),
            None => Err(DeYamlError::Custom("Failed to parse scalar".to_string())),
        }
    }
}

impl<'de, R> de::Deserializer<'de> for &mut YamIterDeserializer<'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128
        u8 u16 u32 u64 u128
        bool f32 f64  char str string
        unit_struct newtype_struct tuple
        tuple_struct ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.skip_doc() {
            Some(YamEvent::MapStart(_, _)) => self.deserialize_map(visitor),
            Some(YamEvent::SeqStart(_, _)) => self.deserialize_seq(visitor),
            Some(YamEvent::Scalar(scalar_value)) => {
                self.skip();
                YamIterDeserializer::<R>::resolve_scalar(scalar_value, visitor)
            }
            _ => Err(DeYamlError::Custom(
                "Expected a scalar, map or a sequence".to_string(),
            )),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.skip_doc() {
            Some(YamEvent::Scalar(ScalarValue { value, .. })) => {
                self.skip();
                let decoded =
                    binary::decode_as_base64(&value).map_err(DeYamlError::Base64DecodeError)?;
                visitor.visit_bytes(&decoded)
            }
            _ => Err(DeYamlError::ExpectedBytes),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ev = self.skip_doc();
        match ev {
            Some(YamEvent::Scalar(ScalarValue { value, .. })) => {
                self.skip();
                let decoded =
                    binary::decode_as_base64(&value).map_err(DeYamlError::Base64DecodeError)?;
                visitor.visit_bytes(&decoded)
            }
            _ => Err(DeYamlError::ExpectedBytes),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        if self.peek_null() {
            self.skip();
            visitor.visit_unit()
        } else {
            Err(DeYamlError::ExpectedNull)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if !matches!(self.skip_doc(), Some(YamEvent::SeqStart(_, _))) {
            return Err(DeYamlError::UnExpectedEvent(
                self.last_event.as_simple_str(),
                "SeqStart",
            ));
        }
        self.skip();
        let val = visitor.visit_seq(SeqCollection::new_seq(self))?;
        if !matches!(self.last_event, YamEvent::SeqEnd)
            && !matches!(self.next_el(), Some(YamEvent::SeqEnd))
        {
            return Err(DeYamlError::UnExpectedEvent(
                self.last_event.as_simple_str(),
                "SeqEnd",
            ));
        }
        self.skip();
        Ok(val)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if !matches!(self.skip_doc(), Some(YamEvent::MapStart(_, _))) {
            return Err(DeYamlError::UnExpectedEvent(
                self.last_event.as_simple_str(),
                "MapStart",
            ));
        }
        self.skip();
        let val = visitor.visit_map(SeqCollection::new_map(self))?;
        if !matches!(self.last_event, YamEvent::MapEnd)
            && !matches!(self.next_el(), Some(YamEvent::MapEnd))
        {
            return Err(DeYamlError::UnExpectedEvent(
                self.last_event.as_simple_str(),
                "MapEnd",
            ));
        }
        self.skip();
        Ok(val)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ev = self.skip_doc();
        match ev {
            Some(YamEvent::MapStart(_, _)) => {
                self.skip();
                let value = visitor.visit_map(SeqCollection::new_map(self))?;

                if !matches!(self.next_el(), Some(YamEvent::MapEnd)) {
                    return Err(DeYamlError::UnExpectedEvent(
                        self.last_event.as_simple_str(),
                        "MapEnd",
                    ));
                }
                Ok(value)
            }
            _ => Err(DeYamlError::Custom(
                "Expected struct to start with MapStart".to_string(),
            )),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
                    return Err(DeYamlError::UnExpectedEvent(
                        self.last_event.as_simple_str(),
                        "MapEnd",
                    ));
                }
                Ok(value)
            }
            _ => Err(DeYamlError::Custom("Expected enum".to_string())),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
}

impl<'de, R> YamIterDeserializer<'de, R>
where
    R: Source,
{
    fn skip_doc<'a>(&'a mut self) -> Option<YamEvent<'de>> {
        let next_el = self.next_el();
        match next_el {
            Some(YamEvent::DocStart) => {
                self.skip();
                self.next_el()
            }
            ev => ev,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DeYamlError {
    ParserError,
    Custom(String),
    ExpectedNull,
    ExpectedBytes,
    Base64DecodeError(DecodeError),
    UnExpectedEvent(&'static str, &'static str),
}

impl StdError for DeYamlError {}

pub(crate) const PARSER_ERROR: &str = "Invalid Yaml";
pub(crate) const EXPECTED_NULL: &str = "Expected null";

impl Display for DeYamlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DeYamlError::ParserError => write!(f, "{PARSER_ERROR}")?,
            DeYamlError::Custom(msg) => write!(f, "{msg}")?,
            DeYamlError::ExpectedNull => write!(f, "{EXPECTED_NULL}")?,
            DeYamlError::ExpectedBytes => write!(f, "Expected to find bytes")?,
            DeYamlError::Base64DecodeError(msg) => write!(f, "Base 64 Decoder error {msg}")?,
            DeYamlError::UnExpectedEvent(actual, expected) => {
                write!(f, "Expected {expected} event instead got {actual} event")?;
            }
        }
        Ok(())
    }
}

impl de::Error for DeYamlError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        let info = format!("{msg}");
        DeYamlError::Custom(info)
    }
}

struct SeqCollection<'a, 'de: 'a, R>
where
    R: Source,
{
    iter: &'a mut YamIterDeserializer<'de, R>,
    // map: bool,
}

impl<'a, 'de, R> SeqCollection<'a, 'de, R>
where
    R: Source,
{
    fn new_seq(iter: &'a mut YamIterDeserializer<'de, R>) -> Self {
        SeqCollection { iter }
    }

    fn new_map(iter: &'a mut YamIterDeserializer<'de, R>) -> Self {
        SeqCollection { iter }
    }
}

impl<'de, R> SeqAccess<'de> for SeqCollection<'_, 'de, R>
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
            Some(YamEvent::DocEnd | YamEvent::StreamEnd) | None => Err(
                DeYamlError::UnExpectedEvent(self.iter.last_event.as_simple_str(), "SeqEnd"),
            ),
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }
}

impl<'de, R> MapAccess<'de> for SeqCollection<'_, 'de, R>
where
    R: Source,
{
    type Error = DeYamlError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let ev = self.iter.next_el();
        match ev {
            Some(YamEvent::MapEnd) => Ok(None),
            Some(YamEvent::DocEnd | YamEvent::StreamEnd) | None => Err(
                DeYamlError::UnExpectedEvent(self.iter.last_event.as_simple_str(), "MapEnd"),
            ),
            Some(_) => seed.deserialize(&mut *self.iter).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.iter)?;
        self.iter.skip();
        Ok(val)
    }
}

struct Enum<'a, 'de: 'a, R>
where
    R: Source,
{
    de: &'a mut YamIterDeserializer<'de, R>,
}

impl<'a, 'de, R> Enum<'a, 'de, R>
where
    'de: 'a,
    R: Source,
{
    fn new(de: &'a mut YamIterDeserializer<'de, R>) -> Self {
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
        de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut *self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut *self.de, visitor)
    }
}

#[cfg(test)]
mod test {
    use crate::serde::de::{DeYamlError, EXPECTED_NULL};
    use crate::serde::from_str;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn test_wrong_de_errors() {
        let evil_input = "true";
        assert_eq!(
            from_str::<i8>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected i8"#.to_string()),
        );
        assert_eq!(
            from_str::<i16>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected i16"#.to_string()),
        );
        assert_eq!(
            from_str::<i32>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected i32"#.to_string()),
        );
        assert_eq!(
            from_str::<i64>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected i64"#.to_string()),
        );
        assert_eq!(
            from_str::<u8>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected u8"#.to_string()),
        );
        assert_eq!(
            from_str::<u16>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected u16"#.to_string()),
        );
        assert_eq!(
            from_str::<u32>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected u32"#.to_string()),
        );
        assert_eq!(
            from_str::<u64>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected u64"#.to_string()),
        );
        assert_eq!(
            from_str::<f32>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected f32"#.to_string()),
        );
        assert_eq!(
            from_str::<f64>(evil_input).unwrap_err(),
            DeYamlError::Custom(r#"invalid type: boolean `true`, expected f64"#.to_string()),
        );

        assert_eq!(
            format!("{}", from_str::<f64>("").unwrap_err()),
            "Expected a scalar, map or a sequence".to_string(),
        );
    }

    #[test]
    fn test_visit_unit() {
        assert_eq!(
            format!("{}", from_str::<()>("zzz").unwrap_err()),
            EXPECTED_NULL,
        );
    }

    use crate::node::Yaml;
    use crate::prelude::YamlDocAccess;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;
    use serde::Deserialize;

    #[test]
    fn test_nested_collections() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: String,
            b: Vec<f32>,
        }

        let ex = r#"{ a: "x",  b: [2.0, 3.1, -1.2] }"#;
        let deserialized: Ex = crate::from_str(ex).unwrap();
        assert_eq!(deserialized.a, "x");
        assert_eq!(deserialized.b, vec![2.0, 3.1, -1.2]);
    }

    #[test]
    fn test_schemas_null() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: Option<String>,
        }
        let input = r"a: !!null";
        let deserialized: Ex = crate::from_str(input).unwrap();
        assert_eq!(deserialized.a, None);

        let input = r"a: !!null null";
        let deserialized2: Ex = crate::from_str(input).unwrap();
        assert_eq!(deserialized2.a, None);
    }

    #[test]
    fn test_schemas_float() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: Option<f32>,
        }
        let input = r"a: 3";
        let deserialized: Yaml = crate::from_str(input).unwrap();
        assert_eq!(deserialized["a"], 3);
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(f32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let j = r#"{ Unit }"#;
        let expected = E::Unit;
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{ Newtype: 1 }"#;
        let expected = E::Newtype(1.0);
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{"Tuple":[1,2]}"#;
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{"Struct":{"a":1}}"#;
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, crate::from_str(j).unwrap());
    }

    #[test]
    fn test_doc_example1() {
        #[derive(Deserialize, PartialEq, Debug)]
        pub struct Person {
            name: String,
            address: Option<String>,
            phones: Vec<String>,
        }

        let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

        let person: Person = crate::from_str(data).expect("Yaml to be parsed!");
        assert_eq!(person.name, "John Smith");
        assert_eq!(person.address, None);
        assert_eq!(person.phones, vec!["+44 1234567", "+32 2345678"]);
    }

    #[test]
    fn test_doc_example2() {
        let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

        let person = Yaml::load_single(data).expect("Yaml to be parsed!");
        assert_eq!(person["name"].as_str(), Some("John Smith"));
        assert_eq!(person["address"].as_str(), None);
        assert_eq!(person["phones"][1].as_str(), Some("+32 2345678"));
    }
}
