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

    pub fn write_double_quote(&mut self, str: &str) -> Result<(), Error> {
        write!(self.writer, "\"")?;
        escape_double_quotes(&mut self.writer, str)?;
        write!(self.writer, "\"")?;
        Ok(())
    }
}

fn peekz_byte(array: &[u8], pos: usize) -> u8 {
    if pos < array.len() { array[pos] } else { 0 }
}

fn decode_hex<W: Write>(writer: &mut W, digit_slice: &[u8]) -> Result<(), Error> {
    if !digit_slice.iter().all(u8::is_ascii_hexdigit) {
        writer.write_char('\u{FFFD}')?;
        return Ok(());
    }

    let code_point = digit_slice
        .iter()
        .map(|x| match *x {
            n @ b'0'..=b'9' => n - b'0',
            a @ b'a'..=b'f' => a - b'a' + 10,
            a @ b'A'..=b'F' => a - b'A' + 10,
            _ => 0u8,
        })
        .fold(0u32, |acc, digit| (acc << 4) + u32::from(digit));
    match code_point {
        // YAML has special escape rules for certain values
        // See more in https://yaml.org/spec/1.2.2/#57-escaped-characters
        0 => writer.write_char('\u{FFFD}')?,
        0x07 => writer.write_str("\\a")?,
        0x08 => writer.write_str("\\b")?,
        0x09 => writer.write_str("\\t")?,
        0x0A => writer.write_str("\\n")?,
        0x0B => writer.write_str("\\v")?,
        0x0C => writer.write_str("\\f")?,
        0x0D => writer.write_str("\\r")?,
        0x1B => writer.write_str("\\e")?,
        0x20 => writer.write_str(" ")?,
        0x22 => writer.write_str("\\\"")?,
        0x2F => writer.write_str("\\/")?,
        0x5C => writer.write_str("\\\\")?,
        0x85 => writer.write_str("\\N")?,
        0xA0 => writer.write_str("\\_")?,
        0x2028 => writer.write_str("\\L")?,
        0x2029 => writer.write_str("\\P")?,
        _ => {
            let encode_char = char::from_u32(code_point);
            if let Some(encode_char) = encode_char {
                return writer.write_char(encode_char);
            }
        }
    }

    Ok(())
}

pub(crate) fn escape_double_quotes<W: Write>(writer: &mut W, value: &str) -> Result<(), Error> {
    let bytes = value.as_bytes();

    let (mut old_pos, mut pos) = (0, 0);
    while pos < bytes.len() {
        let byte_char = bytes[pos];
        let peek_char = peekz_byte(bytes, pos + 1);
        match (byte_char, peek_char) {
            (b'\\', b't' | b'r' | b'n') => {
                // TODO normalize `\r\n` into `\n`
                pos += 2;
            }

            (b'\t', _) => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\t")?;
                pos += 1;
                old_pos = pos;
            }
            (b'\r', b'\n') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\n")?;
                pos += 2;
                old_pos = pos;
            }
            (b'\n', ..) => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\n")?;
                pos += 1;
                old_pos = pos;
            }
            (b'\\', b'x') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 4])?;
                pos += 4;
                old_pos = pos;
            }
            (b'\\', b'u') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 6])?;
                pos += 6;
                old_pos = pos;
            }
            (b'\\', b'U') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 8])?;
                pos += 8;
                old_pos = pos;
            }
            _ => {
                pos += 1;
            }
        }
    }
    if pos != old_pos {
        let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
        writer.write_str(prev_str)?;
    }
    Ok(())
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
    fn serialize_nums<T: Display>(&mut self, value: T) -> Result<(), Error> {
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
        self.serialize_nums(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_nums(v)
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
