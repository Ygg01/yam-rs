use crate::node::{Yaml, YamlData, YamlScalar};
use crate::prelude::{ToMut, ToMutStr, YamlEntry};
use alloc::string::String;
use alloc::vec::Vec;
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
pub struct YamPhantomSerializer<'de, F, I, S>(&'de PhantomData<()>, PhantomData<(F, I, S)>);

#[doc(hidden)]
pub struct YamSeqSerializer<'de, F, I, S> {
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamMapSerializer<'de, F, I, S> {
    seq: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

#[doc(hidden)]
pub struct YamTupleVariantSerializer<'de, F, I, S> {
    variant: String,
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamStructVariantSerializer<'de, F, I, S> {
    variant: String,
    seq: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

impl<'de, F, I, S> Serializer for YamPhantomSerializer<'de, F, I, S> {
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
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeSeq for YamSeqSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeTuple for YamSeqSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeTupleStruct for YamSeqSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeMap for YamMapSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeStruct for YamMapSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeTupleVariant for YamTupleVariantSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'de, F, I, S> SerializeStructVariant for YamStructVariantSerializer<'de, F, I, S> {
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
