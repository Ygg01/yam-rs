use alloc::borrow::Cow;
use core::fmt::{Debug, Display, Error, Write};
use serde_core::ser::{SerializeMap, SerializeStructVariant};
use serde_core::{Serialize, ser};

#[derive(Debug)]
pub struct YamSerializer<W> {
    /// This string starts empty and JSON is appended as values are serialized.
    pub(crate) writer: W,
    /// Pretty configuration option for formatting
    pub(crate) formatter: PrettyFormatter,
}

impl<W> YamSerializer<W>
where
    W: Write,
{
    #[inline]
    pub fn new_pretty(writer: W, formatter: PrettyFormatter) -> Self {
        YamSerializer { writer, formatter }
    }
}

impl<W> YamSerializer<W> {
    #[inline]
    pub fn new_simple(writer: W) -> Self {
        YamSerializer {
            writer,
            formatter: PrettyFormatter::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum NullFormat {
    #[default]
    /// Null that has corresponds to JSON null
    /// ```yaml
    /// example: null
    /// ```
    JsonNull,
    /// Null that has a schema built in.
    /// ```yaml
    /// example: !!null null
    /// ```
    TaggedYaml,
    /// Null that's just an empty yaml.
    /// ```yaml
    /// example: # the value of null key is null.
    /// ```
    Plain,
    /// Null used in Yaml 1.1 i.e.
    /// ```yaml
    /// example: ~
    /// ```
    OldYaml,
}

#[derive(Debug)]
pub struct PrettyFormatter {
    current_indent: usize,

    /// Pretty YAML-like format
    pub yaml_format: bool,

    /// Limit depth
    pub depth_limit: usize,

    /// Indentation string
    pub indentor: Cow<'static, str>,

    /// New line string
    pub new_line: Cow<'static, str>,

    /// How to format null
    null_format: Cow<'static, str>,
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self {
            current_indent: 0,
            yaml_format: false,
            depth_limit: 0,
            indentor: Cow::Borrowed(""),
            new_line: Cow::Borrowed(""),
            null_format: Cow::Borrowed(""),
        }
    }
}

impl PrettyFormatter {
    pub fn pretty() -> Self {
        Self {
            current_indent: 0,
            yaml_format: true,
            depth_limit: 10,
            indentor: Cow::Borrowed("  "),
            new_line: Cow::Borrowed("\n"),
            null_format: Cow::Borrowed("null"),
        }
    }

    #[inline]
    fn set_null_format(&mut self, fmt: NullFormat) {
        self.null_format = match fmt {
            NullFormat::JsonNull => Cow::Borrowed("null"),
            NullFormat::TaggedYaml => Cow::Borrowed("!!null null"),
            NullFormat::Plain => Cow::Borrowed(""),
            NullFormat::OldYaml => Cow::Borrowed("~"),
        };
    }
}

impl<W> YamSerializer<W>
where
    W: Write,
{
    fn serialize_sint<T: Display>(&mut self, value: T) -> Result<(), Error> {
        write!(self.writer, "{value}")?;
        Ok(())
    }
}

impl<W> ser::Serializer for &mut YamSerializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let str = if v { "true" } else { "false" };
        self.writer.write_str(str)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_sint(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if v.is_nan() && v.is_sign_negative() {
            write!(self.writer, "-")?;
        }

        write!(self.writer, "{v}")?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if v.is_nan() && v.is_sign_negative() {
            write!(self.writer, "-")?;
        }

        write!(self.writer, "{v}")?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let string_chr = if v == '\'' { '"' } else { '\'' };

        self.writer.write_char(string_chr)?;
        self.writer.write_char(v)?;
        self.writer.write_char(string_chr)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
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
        self.writer.write_str(&self.formatter.null_format)
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

impl<W> ser::SerializeSeq for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> ser::SerializeMap for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> ser::SerializeTuple for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> ser::SerializeTupleStruct for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> SerializeStructVariant for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> ser::SerializeTupleVariant for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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

impl<W> ser::SerializeStruct for &mut YamSerializer<W> {
    type Ok = ();
    type Error = Error;

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
