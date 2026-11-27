use crate::binary::encode_as_base64;
use crate::node::{Yaml, YamlData, YamlScalar};
use crate::parsing::Tag;
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use crate::serde::to_yaml;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Error};
use core::marker::PhantomData;
use serde_core::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde_core::{Serialize, Serializer};

impl<'de, FLOAT, INT, STR> Serialize for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
    STR: Clone + ToMutStr<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            YamlData::BadValue | YamlData::Scalar(YamlScalar::Null(_)) => {
                serializer.serialize_unit()
            }
            YamlData::Scalar(YamlScalar::String(s)) => serializer.serialize_str(s.as_str()),
            YamlData::Scalar(YamlScalar::FloatingPoint(f)) => {
                serializer.serialize_f64(f.as_owned())
            }
            YamlData::Scalar(YamlScalar::Integer(i)) => serializer.serialize_i64(i.as_owned()),
            YamlData::Scalar(YamlScalar::Bool(n)) => serializer.serialize_bool(*n),
            YamlData::Sequence(seq) => seq.serialize(serializer),
            YamlData::Mapping(map) => {
                use serde_core::ser::SerializeMap;
                let mut map_serializer = serializer.serialize_map(Some(map.len()))?;
                for entry in map {
                    map_serializer.serialize_entry(&entry.key, &entry.value)?;
                }
                map_serializer.end()
            }
            YamlData::Tagged(_, node) => node.serialize(serializer),
            YamlData::Alias(_) => todo!("Implement alias"),
        }
    }
}

#[doc(hidden)]
#[allow(dead_code)]
pub struct YamPhantomSerializer<'de, F, I, S>(&'de PhantomData<()>, PhantomData<(F, I, S)>);

impl<'de, F, I, S> Default for YamPhantomSerializer<'de, F, I, S> {
    fn default() -> Self {
        YamPhantomSerializer(&PhantomData, PhantomData)
    }
}

#[doc(hidden)]
pub struct YamSeqSerializer<'de, F, I, S> {
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamMapSerializer<'de, F, I, S> {
    entries: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

#[doc(hidden)]
pub struct YamTupleVariantSerializer<'de, F, I, S> {
    variant: String,
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamStructVariantSerializer<'de, F, I, S> {
    variant: String,
    entries: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

impl<'de, F, I, S> Serializer for YamPhantomSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;
    type SerializeSeq = YamSeqSerializer<'de, F, I, S>;
    type SerializeTuple = YamSeqSerializer<'de, F, I, S>;
    type SerializeTupleStruct = YamSeqSerializer<'de, F, I, S>;
    type SerializeTupleVariant = YamTupleVariantSerializer<'de, F, I, S>;
    type SerializeMap = YamMapSerializer<'de, F, I, S>;
    type SerializeStruct = YamMapSerializer<'de, F, I, S>;
    type SerializeStructVariant = YamStructVariantSerializer<'de, F, I, S>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if let Ok(conv) = i64::try_from(v) {
            Ok(Yaml::from_integer(conv))
        } else {
            Err(Error)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_float(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut str = String::with_capacity(4);
        str.push(v);
        Ok(Yaml::from_string(str))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_string(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let encode_bytes = encode_as_base64(v);
        Ok(Yaml(YamlData::Tagged(
            Cow::Owned(Tag::binary_schema()),
            Box::new(Yaml::from_string(encode_bytes)),
        )))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::null())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        let mut mapping = Vec::new();
        let key = Yaml::from_str(name);
        let value = Yaml::null();
        mapping.push(YamlEntry::new(key, value));

        Ok(Yaml(YamlData::Mapping(mapping)))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let mut mapping = Vec::new();
        let key = Yaml::from_str(variant);
        let value = Yaml::null();
        mapping.push(YamlEntry::new(key, value));

        Ok(Yaml(YamlData::Mapping(mapping)))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut mapping = Vec::new();
        let key = Yaml::from_str(variant);
        let value = to_yaml(value)?;
        mapping.push(YamlEntry::new(key, value));

        Ok(Yaml(YamlData::Mapping(mapping)))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(YamSeqSerializer {
            seq: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(YamTupleVariantSerializer {
            variant: variant.to_string(),
            seq: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(YamMapSerializer {
            entries: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(YamMapSerializer {
            entries: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(YamStructVariantSerializer {
            variant: variant.to_string(),
            entries: Vec::with_capacity(len),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}

impl<'de, F, I, S> SerializeSeq for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = match to_yaml(value) {
            Ok(x) => x,
            Err(_) => return Err(core::fmt::Error),
        };
        self.seq.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeTuple for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeTupleStruct for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeMap for YamMapSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let key = to_yaml(key)?;
        let val = to_yaml(value)?;
        self.entries.push(YamlEntry::new(key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Mapping(self.entries)))
    }
}

impl<'de, F, I, S> SerializeStruct for YamMapSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Mapping(self.entries)))
    }
}

impl<'de, F, I, S> SerializeTupleVariant for YamTupleVariantSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let val = to_yaml(value)?;
        self.seq.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let entry = YamlEntry::new(
            Yaml::from_string(self.variant),
            Yaml(YamlData::Sequence(self.seq)),
        );

        Ok(Yaml(YamlData::Mapping(vec![entry])))
    }
}

impl<'de, F, I, S> SerializeStructVariant for YamStructVariantSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = Yaml::from_string(key);
        let value = to_yaml(value)?;
        self.entries.push(YamlEntry::new(key, value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let entry = YamlEntry::new(
            Yaml::from_string(self.variant),
            Yaml(YamlData::Mapping(self.entries)),
        );
        Ok(Yaml(YamlData::Mapping(vec![entry])))
    }
}
