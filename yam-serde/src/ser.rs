use crate::binary;
use crate::escape_str::{escape_double_quotes, peekz_byte};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Error, Write};
use core::ops::Deref;
use ser::{SerializeSeq, Serializer};
use serde_core::ser::{SerializeMap, SerializeStructVariant};
use serde_core::{Serialize, ser};
use unicode_segmentation::UnicodeSegmentation;

trait YamlWhitespace {
    fn is_splittable_ws(&self) -> bool;
    fn is_last_char_splittable_ws(&self) -> bool;
}

impl YamlWhitespace for str {
    fn is_splittable_ws(&self) -> bool {
        self.bytes().all(|c| c == b' ' || c == b'\n')
    }

    fn is_last_char_splittable_ws(&self) -> bool {
        self.bytes()
            .last()
            .map(|c| c == b' ' || c == b'\n')
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct YamSerializer<W> {
    /// This string starts empty and JSON is appended as values are serialized.
    pub(crate) writer: W,
    pub(crate) position: usize,
    pub(crate) current_depth: usize,
    /// Pretty configuration option for formatting
    pub(crate) formatter: PrettyFormatterConfig,
    pub(crate) indentor_len: usize,
}

impl<W> YamSerializer<W>
where
    W: Write,
{
    #[inline]
    pub fn new_pretty(writer: W, formatter: PrettyFormatterConfig) -> Self {
        let indentor_size = formatter.indentor.graphemes(true).count();
        YamSerializer {
            writer,
            formatter,
            position: 0,
            current_depth: 0,
            indentor_len: indentor_size,
        }
    }

    #[inline]
    pub(crate) fn use_complex_form(&self) -> bool {
        self.current_depth <= self.formatter.depth_limit
    }

    pub(crate) fn write_obj_start(&mut self) -> Result<(), Error> {
        if self.use_complex_form() {
            self.write_indentors(self.current_depth)?;
        } else {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    pub(crate) fn write_obj_end(&mut self) -> Result<(), Error> {
        if !self.use_complex_form() {
            self.write_ascii("}")?;
        }
        Ok(())
    }

    pub(crate) fn write_seq_start(&mut self) -> Result<(), Error> {
        if self.use_complex_form() {
            self.write_indentors(self.current_depth)?;
        } else {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    pub(crate) fn write_seq_end(&mut self) -> Result<(), Error> {
        if !self.use_complex_form() {
            self.write_ascii("}")?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), Error> {
        let res = self.writer.write_char(c);
        self.position += 1;
        res
    }

    fn write_string(&mut self, str: &str) -> Result<(), Error> {
        let res = self.writer.write_str(str);
        self.position += str.graphemes(true).count();
        res
    }

    fn write_ascii(&mut self, str: &str) -> Result<(), Error> {
        let res = self.writer.write_str(str);
        self.position += str.len();
        res
    }

    fn write_nl(&mut self) -> Result<(), Error> {
        let res = self.writer.write_char('\n');
        self.position = 0;
        res
    }

    fn write_indent(&mut self) -> Result<(), Error> {
        let res = self.writer.write_char('\n');
        self.position = 0;
        res
    }

    fn is_time_to_split(&self, buff_len: usize, word_len: usize) -> bool {
        buff_len + word_len > self.formatter.pref_string_length
    }

    fn write_indentors(&mut self, indent: usize) -> Result<(), Error> {
        for _ in 0..indent {
            self.writer.write_str(&self.formatter.indentor)?;
        }
        self.position += indent * self.indentor_len;
        Ok(())
    }

    fn write_single_line(&mut self, fence: &str, escaped_str: &str) -> Result<(), Error> {
        self.write_string(fence)?;
        self.write_string(escaped_str)?;
        self.write_string(fence)?;
        self.position += 2 * fence.len() + escaped_str.graphemes(true).count();
        Ok(())
    }

    fn write_block_string(&mut self, str: &str) -> Result<(), Error> {
        let mut string_writer = String::with_capacity(str.len() * 2);
        string_writer.push_str(str);

        self.write_string("|\n")?;
        self.write_string(&string_writer)?;

        self.position += 2 + string_writer.graphemes(true).count();
        Ok(())
    }

    fn line_split_at(&mut self, line_buff: &str, line_split: &str) -> Result<(), Error> {
        let escaped = if line_split == " " { "\n" } else { "\n\n" };
        self.writer.write_str(line_buff)?;
        self.writer.write_str(escaped)?;
        self.write_indentors(self.current_depth)
    }

    fn write_multi_line_string(
        &mut self,
        prefix: &str,
        str: &str,
        suffix: &str,
    ) -> Result<(), Error> {
        self.write_ascii(prefix)?;

        let mut line_buff = String::with_capacity(self.formatter.pref_string_length + 20);
        let mut line_buff_len = 0;
        let word_bounds = str
            .split_word_bound_indices()
            .map(|(_, word)| (word, word.graphemes(true).count()))
            .collect::<Vec<(&str, usize)>>();

        for (word, grapheme_len) in word_bounds {
            if self.is_time_to_split(self.position, grapheme_len) {
                let word_is_splittable = word.is_splittable_ws();
                let line_buff_is_splittable = line_buff.is_last_char_splittable_ws();

                if line_buff_is_splittable {
                    // Try to split line on existing buffer
                    let (line, nl) = line_buff.split_at(line_buff.len() - 1);
                    self.line_split_at(line, nl)?;

                    // Set current buffer to current word
                    line_buff.clear();
                    line_buff.push_str(word);
                    line_buff_len = grapheme_len;
                } else if word_is_splittable {
                    // Try to split line on word
                    let (front, nl) = word.split_at(0);
                    self.line_split_at(&line_buff, nl)?;

                    line_buff.clear();
                    line_buff.push_str(front);
                    line_buff_len = front.len();
                } else {
                    // Write the word to buffer
                    line_buff.push_str(word);
                    line_buff_len += grapheme_len;
                }
            } else {
                line_buff.push_str(word);
                line_buff_len += grapheme_len;
            }
        }
        self.writer.write_str(&line_buff)?;
        self.write_ascii(suffix)?;
        Ok(())
    }
}

pub(crate) fn escape_single_quotes<W: Write>(writer: &mut W, value: &str) -> Result<(), Error> {
    let bytes = value.as_bytes();

    let (mut old_pos, mut pos) = (0, 0);
    while pos < bytes.len() {
        let byte_char = bytes[pos];
        let peek_char = peekz_byte(bytes, pos + 1);
        match (byte_char, peek_char) {
            (b'\'', _) => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "''")?;
                pos += 1;
                old_pos = pos;
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
            formatter: PrettyFormatterConfig::default(),
            position: 0,
            indentor_len: 0,
            current_depth: 0,
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
pub struct PrettyFormatterConfig {
    /// Pretty YAML-like format
    pub yaml_format: bool,

    /// Limit depth
    pub depth_limit: usize,

    /// Preferred string length
    pub pref_string_length: usize,

    /// Indentation string
    pub indentor: Cow<'static, str>,

    /// New line string
    pub new_line: Cow<'static, str>,

    /// How to format null
    null_format: Cow<'static, str>,
}

impl Default for PrettyFormatterConfig {
    fn default() -> Self {
        Self {
            yaml_format: false,
            depth_limit: 0,
            pref_string_length: 80,
            indentor: Cow::Borrowed(""),
            new_line: Cow::Borrowed(""),
            null_format: Cow::Borrowed(""),
        }
    }
}

impl PrettyFormatterConfig {
    pub fn pretty() -> Self {
        Self {
            yaml_format: true,
            depth_limit: 10,
            pref_string_length: 80,
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

#[doc(hidden)]
#[derive(Eq, PartialEq)]
pub enum CompoundState {
    Empty,
    First,
    Rest,
}

#[doc(hidden)]
pub enum Compound<'a, W> {
    Map {
        ser: &'a mut YamSerializer<W>,
        state: CompoundState,
    },
    Seq {
        ser: &'a mut YamSerializer<W>,
        state: CompoundState,
    },
}

impl<'a, W> Serializer for &'a mut YamSerializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let str = if v { "true" } else { "false" };
        self.writer.write_str(str)?;
        self.position += str.len();
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
        if !self.use_complex_form() {
            let escaped_single = {
                let mut str_writer = String::with_capacity(v.len());
                escape_double_quotes(&mut str_writer, v)?;
                str_writer
            };
            self.write_single_line("\"", &escaped_single)?;
        } else {
            self.write_multi_line_string("\"", v, "\"")?;
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.write_ascii("!!binary")?;
        let mut encoded_bytes = binary::encode_as_base64(v);
        if !self.use_complex_form() {
            self.write_ascii("\"")?;
            self.write_ascii(&encoded_bytes)?;
            self.write_ascii("\"")?;
        } else {
            self.write_ascii("|\n")?;

            while !encoded_bytes.is_empty() {
                self.write_indentors(self.current_depth)?;
                let remaining_byte = self
                    .formatter
                    .pref_string_length
                    .saturating_sub(self.position);
                let write = encoded_bytes.split_off(remaining_byte);
                self.write_ascii(&write)?;
            }
            self.write_ascii(&encoded_bytes)?;
        }
        Ok(())
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
        self.writer.write_str(&self.formatter.null_format)?;
        self.position += self.formatter.null_format.len();
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.write_obj_start()?;
        self.write_obj_end()?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_single_line("\"", variant)?;
        Ok(())
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
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'a, W> SerializeSeq for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> SerializeMap for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Compound::Map { ser, .. } => key.serialize(&mut **ser)?,
            Compound::Seq { ser, .. } => key.serialize(&mut **ser)?,
        }

        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> SerializeStructVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ser::PrettyFormatterConfig;
    use crate::to_pretty_string;
    use alloc::string::ToString;

    const MULTI_LINE_STRING1_ACTUAL: &str = "One quick brown fox jumps over the lazy dog";
    const MULTI_LINE_STRING1_EXPECTED: &str = r#""One quick
brown fox
jumps over
the lazy
dog""#;
    #[test]
    fn test_multiline_string() {
        let formatter = {
            let mut x = PrettyFormatterConfig::pretty();
            x.pref_string_length = 10;
            x
        };
        let result = to_pretty_string(&MULTI_LINE_STRING1_ACTUAL, formatter);
        assert_eq!(result, Ok(MULTI_LINE_STRING1_EXPECTED.to_string()));
    }
}
