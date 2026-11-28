use crate::node::{Yaml, YamlData, YamlScalar};
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use crate::serde::de::DeYamlError;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde_core::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde_core::{Deserialize, Deserializer, forward_to_deserialize_any};

impl<'de, FLOAT, INT, STR> Deserialize<'de> for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64> + PartialEq,
    INT: Copy + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'de> + PartialEq,
{
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YamlVisitor<F, I, S>(PhantomData<(F, I, S)>);
        impl<F, I, S> YamlVisitor<F, I, S> {
            fn new() -> Self {
                YamlVisitor(PhantomData)
            }
        }

        impl<'de, F, I, S> Visitor<'de> for YamlVisitor<F, I, S>
        where
            F: Copy + ToMut<f64> + PartialEq,
            I: Copy + ToMut<i64> + PartialEq,
            S: Clone + ToMutStr<'de> + PartialEq,
        {
            type Value = Yaml<'de, F, I, S>;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
                formatter.write_str("any valid YAML value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_bool(v))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_char(v))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_string(v.to_string()))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_str(v))
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::null())
            }

            fn visit_seq<VS>(self, mut seq_visitor: VS) -> Result<Self::Value, VS::Error>
            where
                VS: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq_visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Yaml(YamlData::Sequence(vec)))
            }

            fn visit_map<A>(self, mut map_visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(key) = map_visitor.next_key()? {
                    let val = map_visitor.next_value()?;
                    let entry = YamlEntry::new(key, val);
                    vec.push(entry);
                }

                Ok(Yaml(YamlData::Mapping(vec)))
            }
        }

        deserializer.deserialize_any(YamlVisitor::new())
    }
}

struct YamSeqDeserializer<'yaml, F, I, S> {
    iter: vec::IntoIter<Yaml<'yaml, F, I, S>>,
}

impl<'yaml, F, I, S> YamSeqDeserializer<'yaml, F, I, S> {
    pub(crate) fn new(vec: Vec<Yaml<'yaml, F, I, S>>) -> Self {
        YamSeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de: 'yaml, 'yaml, F, I, S> SeqAccess<'de> for YamSeqDeserializer<'yaml, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'de> + PartialEq,
{
    type Error = DeYamlError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(val) => seed.deserialize(val).map(Some),
            None => Ok(None),
        }
    }
}

struct YamMapDeserializer<'yaml, F, I, S> {
    iter: vec::IntoIter<YamlEntry<'yaml, Yaml<'yaml, F, I, S>>>,
    value: Option<Yaml<'yaml, F, I, S>>,
}

impl<'yaml, F, I, S> YamMapDeserializer<'yaml, F, I, S> {
    pub(crate) fn new(vec: Vec<YamlEntry<'yaml, Yaml<'yaml, F, I, S>>>) -> Self {
        YamMapDeserializer {
            iter: vec.into_iter(),
            value: None,
        }
    }
}

impl<'de: 'yaml, 'yaml, F, I, S> MapAccess<'de> for YamMapDeserializer<'yaml, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'de> + PartialEq,
{
    type Error = DeYamlError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(yaml) => {
                self.value = Some(yaml.value);
                seed.deserialize(yaml.key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(DeYamlError::Custom("value is missing".to_string())),
        }
    }
}

fn visit_sequence<'de: 'yaml, 'yaml, V, F, I, S>(
    array: Vec<Yaml<'yaml, F, I, S>>,
    visitor: V,
) -> Result<V::Value, DeYamlError>
where
    V: Visitor<'de>,
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'de> + PartialEq,
{
    let len = array.len();
    let mut deserializer = YamSeqDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(DeYamlError::InvalidLength(len))
    }
}

fn visit_entries<'de: 'yaml, 'yaml, V, F, I, S>(
    array: Vec<YamlEntry<'yaml, Yaml<'yaml, F, I, S>>>,
    visitor: V,
) -> Result<V::Value, DeYamlError>
where
    V: Visitor<'de>,
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'de> + PartialEq,
{
    let len = array.len();
    let mut deserializer = YamMapDeserializer::new(array);
    let seq = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(DeYamlError::InvalidLength(len))
    }
}

impl<'de: 'yaml, 'yaml, FLOAT, INT, STR> Deserializer<'de> for Yaml<'yaml, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64> + PartialEq,
    INT: Copy + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'de> + PartialEq,
{
    type Error = DeYamlError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            YamlData::BadValue | YamlData::Scalar(YamlScalar::Null(_)) => visitor.visit_unit(),
            YamlData::Scalar(YamlScalar::Bool(v)) => visitor.visit_bool(v),
            YamlData::Scalar(YamlScalar::Integer(v)) => visitor.visit_i64(v.as_owned()),
            YamlData::Scalar(YamlScalar::FloatingPoint(v)) => visitor.visit_f64(v.as_owned()),
            YamlData::Scalar(YamlScalar::String(str)) => visitor.visit_str(str.as_str()),
            YamlData::Sequence(seq) => visit_sequence(seq, visitor),
            YamlData::Mapping(entries) => visit_entries(entries, visitor),
            YamlData::Tagged(_, node) => node.deserialize_any(visitor),
            YamlData::Alias(_) => todo!("To implement"),
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64
        u8 u16 u32 u64
        f32 f64 bool char string
        bytes byte_buf
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => visitor.visit_str(s.as_str()),
            _ => Err(DeYamlError::InvalidType),
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            YamlData::Scalar(YamlScalar::Null(_)) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if !matches!(self, Yaml(YamlData::Scalar(YamlScalar::Null(_)))) {
            return Err(DeYamlError::ExpectedNull);
        }
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            YamlData::Sequence(seq) => visit_sequence(seq, visitor),
            _ => Err(DeYamlError::InvalidType),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            YamlData::Mapping(entries) => visit_entries(entries, visitor),
            _ => Err(DeYamlError::InvalidType),
        }
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
        match self.0 {
            YamlData::Mapping(entries) => visit_entries(entries, visitor),
            YamlData::Sequence(seq) => visit_sequence(seq, visitor),
            _ => Err(DeYamlError::InvalidType),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}
