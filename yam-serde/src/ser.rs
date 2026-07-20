use crate::binary;
use crate::escape_str::{escape_double_quotes, peekz_byte};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Error, Write};
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

#[derive(Debug, Default)]
pub struct YamSerializer<W> {
    /// This string starts empty and JSON is appended as values are serialized.
    pub(crate) writer: W,
    pub(crate) position: u32,
    pub(crate) current_depth: u32,
    pub(crate) block_nesting: u32,
    /// Pretty configuration option for formatting
    pub(crate) formatter: PrettyFormatterConfig,
    pub(crate) indentor_len: u32,
    in_block_form: bool,
    in_key: bool,
}

impl<W> YamSerializer<W>
where
    W: Write,
{
    #[inline]
    pub fn new_pretty(writer: W, formatter: PrettyFormatterConfig) -> Self {
        let indentor_size: u32 = formatter
            .indentor
            .graphemes(true)
            .count()
            .try_into()
            .unwrap_or_default();
        YamSerializer {
            writer,
            formatter,
            position: 0,
            current_depth: 0,
            block_nesting: 0,
            indentor_len: indentor_size,
            in_block_form: true,
            in_key: false,
        }
    }

    pub(crate) fn is_explicit_key(&self) -> bool {
        self.block_nesting > 1
    }

    #[inline]
    pub(crate) fn use_block_form(&mut self) -> bool {
        self.in_block_form = self.current_depth <= self.formatter.block_depth_limit;
        self.in_block_form
    }

    pub(crate) fn begin_object(&mut self) -> Result<(), Error> {
        if self.use_block_form() {
            self.write_indent(self.current_depth)?;
        } else {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn end_object(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
            self.write_ascii("}")?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn begin_sequence(&mut self) -> Result<(), Error> {
        if self.use_block_form() {
            self.write_to_pos(self.position)?;
        } else {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn begin_sequence_value(&mut self, first: bool) -> Result<(), Error> {
        if self.use_block_form() {
            self.write_ascii("- ")
        } else if !first && self.use_block_form() {
            self.write_ascii(",")
        } else {
            Ok(())
        }
    }

    #[inline]
    pub(crate) fn end_sequence_value(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    pub(crate) fn end_sequence(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
            self.write_ascii("]")?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn begin_object_key(&mut self, is_first: bool) -> Result<(), Error> {
        if self.is_explicit_key() {
            self.write_ascii("? ")?;
        } else if is_first {
            self.write_ascii(",")?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn end_object_key(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn begin_object_value(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn end_object_value(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn write_seq_start(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
            self.write_indent(self.current_depth)?;
        } else {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    pub(crate) fn write_seq_end(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
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
        let str_count: u32 = str
            .graphemes(true)
            .count()
            .try_into()
            .expect("Expected less than u32::MAX sized line");
        self.position += str_count;
        res
    }

    /// Writes an ASCII string to the underlying writer and updates the current position.
    ///
    /// # Parameters
    /// - `str`: A reference to the ASCII string that will be written to the underlying writer.
    ///   If the string is not ASCII, this function will cause set position to the wrong value.
    ///
    /// # Returns
    /// - Returns `Ok(())` if the string is successfully written.
    /// - Returns `Err(Error)` if there is an error during the write operation.
    ///
    /// # Side Effects
    /// - Increments the `position` field by the length of the string that was written.
    ///
    /// # Errors
    /// - This function propagates any errors that occur when invoking the `write_str` method on the writer.
    fn write_ascii(&mut self, str: &str) -> Result<(), Error> {
        let res = self.writer.write_str(str);
        self.position += str.len() as u32;
        res
    }

    fn write_nl(&mut self) -> Result<(), Error> {
        let res = self.writer.write_char('\n');
        self.position = 0;
        res
    }

    fn is_time_to_split(&self, buff_len: u32) -> bool {
        self.position + buff_len > self.formatter.pref_string_length
    }

    /// Writes an indented newline to the underlying writer.
    ///
    /// This function appends a newline character (`'\n'`) to the writer
    /// and then writes the specified amount of indentation based on the
    /// configured indentation string and level.
    ///
    /// # Arguments
    ///
    /// * `indent` - The number of indentation levels to write. Each level
    ///   corresponds to the `indentor` string defined in the
    ///   formatter.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the newline and indentation are successfully written.
    /// * `Err(Error)` if writing to the underlying writer fails.
    ///
    /// # Side Effects
    ///
    /// * Updates the `position` field to reflect the new cursor position
    ///   based on the total length of the written indentation.
    ///
    /// # Errors
    ///
    /// Returns an error if the `write_str` operation on the writer fails
    /// (e.g., if the writer encounters an I/O error).
    ///
    /// # Example
    ///
    fn write_indent(&mut self, indent: u32) -> Result<(), Error> {
        self.writer.write_char('\n')?;
        for _ in 0..indent {
            self.writer.write_str(&self.formatter.indentor)?;
        }
        self.position = indent * self.indentor_len;
        Ok(())
    }

    fn write_to_pos(&mut self, pos: u32) -> Result<(), Error> {
        let space_num = pos.saturating_sub(self.position);

        let spaces = " ".repeat(space_num as usize);
        self.writer.write_str(&spaces)?;

        self.position += space_num;
        Ok(())
    }

    fn write_single_line(&mut self, fence: &str, escaped_str: &str) -> Result<(), Error> {
        self.write_string(fence)?;
        self.write_string(escaped_str)?;
        self.write_string(fence)?;
        let grapheme_count: u32 = escaped_str.graphemes(true).count().try_into().unwrap();
        self.position += 2 * (fence.len() as u32) + grapheme_count;
        Ok(())
    }

    fn write_block_string(&mut self, str: &str) -> Result<(), Error> {
        let mut string_writer = String::with_capacity(str.len() * 2);
        string_writer.push_str(str);

        // Write the pipe without updating position
        self.writer.write_char('|')?;
        // then indent the string and write the block.
        self.write_indent(self.current_depth)?;
        self.write_string(&string_writer)?;

        Ok(())
    }

    fn line_split_at(&mut self, line_buff: &str, line_split: &str) -> Result<(), Error> {
        let escaped = if line_split == " " { "" } else { "\n" };
        self.writer.write_str(line_buff)?;
        self.writer.write_str(escaped)?;
        self.write_indent(self.current_depth)
    }

    fn write_multi_line_string(
        &mut self,
        prefix: &str,
        str: &str,
        suffix: &str,
    ) -> Result<(), Error> {
        if self.is_time_to_split(0) {
            self.write_indent(self.current_depth)?;
        }
        self.write_ascii(prefix)?;

        let mut line_buff =
            String::with_capacity((self.formatter.pref_string_length + 20).try_into().unwrap());
        let mut line_buff_grapheme_len = 0;
        let word_bounds = str
            .split_word_bound_indices()
            .map(|(_, word)| (word, word.graphemes(true).count()))
            .collect::<Vec<(&str, usize)>>();

        for (word, grapheme_len) in word_bounds {
            let grapheme_len: u32 = grapheme_len
                .try_into()
                .expect("Word length is larger than u32::MAX");
            if self.is_time_to_split(line_buff_grapheme_len + grapheme_len) {
                let word_is_splittable = word.is_splittable_ws();
                let line_buff_is_splittable = line_buff.is_last_char_splittable_ws();

                if line_buff_is_splittable {
                    // Try to split line on existing buffer
                    let (line, nl) = line_buff.split_at(line_buff.len() - 1);
                    self.line_split_at(line, nl)?;

                    // Set current buffer to current word
                    line_buff.clear();
                    line_buff.push_str(word);
                    line_buff_grapheme_len = grapheme_len;
                } else if word_is_splittable {
                    // Try to split line on word
                    let (front, nl) = word.split_at(0);
                    self.line_split_at(&line_buff, nl)?;

                    line_buff.clear();
                    line_buff.push_str(front);
                    line_buff_grapheme_len = front.len() as u32;
                } else {
                    // Write the word to buffer
                    line_buff.push_str(word);
                    line_buff_grapheme_len += grapheme_len;
                }
            } else {
                line_buff.push_str(word);
                line_buff_grapheme_len += grapheme_len;
            }
        }

        self.writer.write_str(&line_buff)?;
        self.position = line_buff_grapheme_len;
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
            block_nesting: 0,
            in_block_form: true,
            in_key: false,
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
    pub block_depth_limit: u32,

    /// Preferred string length
    pub pref_string_length: u32,

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
            block_depth_limit: 0,
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
            block_depth_limit: 10,
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
        self.position += str.len() as u32;
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
        if !self.use_block_form() {
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
        if !self.use_block_form() {
            self.write_ascii("\"")?;
            self.write_ascii(&encoded_bytes)?;
            self.write_ascii("\"")?;
        } else {
            self.write_ascii("|\n")?;

            while !encoded_bytes.is_empty() {
                self.write_indent(self.current_depth)?;
                let remaining_byte = self
                    .formatter
                    .pref_string_length
                    .saturating_sub(self.position);
                let write = encoded_bytes.split_off(remaining_byte as usize);
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
        self.position += self.formatter.null_format.len() as u32;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.write_ascii("{")?;
        self.write_ascii("}")?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_ascii("{")?;
        self.write_string(variant)?;
        self.write_ascii("}")?;
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
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.begin_object()?;

        self.begin_object_key(true)?;
        self.serialize_str(variant)?;
        self.end_object_key()?;

        self.begin_object_value()?;
        value.serialize(&mut *self)?;
        self.end_object_value()?;

        self.end_object()?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.begin_sequence()?;
        if len == Some(0) {
            self.end_sequence()?;
            Ok(Compound::Seq {
                ser: self,
                state: CompoundState::Empty,
            })
        } else {
            Ok(Compound::Seq {
                ser: self,
                state: CompoundState::First,
            })
        }
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
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.begin_object()?;
        self.begin_object_key(true)?;
        self.serialize_str(name)?;
        self.end_object_key()?;
        self.begin_object_value()?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.begin_object()?;
        if len == Some(0) {
            self.end_object()?;
            Ok(Compound::Map {
                ser: self,
                state: CompoundState::Empty,
            })
        } else {
            Ok(Compound::Map {
                ser: self,
                state: CompoundState::First,
            })
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.begin_object()?;

        self.begin_object_key(true)?;
        self.serialize_str(_variant)?;
        self.begin_object_key(false)?;

        self.begin_object_value()?;
        self.serialize_map(Some(len))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
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

impl<'a, W> SerializeSeq for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Compound::Map { ser, state } | Compound::Seq { ser, state } => {
                ser.begin_sequence_value(*state == CompoundState::First)?;
                *state = CompoundState::Rest;
                value.serialize(&mut **ser)?;
                ser.end_sequence_value()
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Compound::Map { ser, state } | Compound::Seq { ser, state } => match state {
                CompoundState::Empty => Ok(()),
                _ => ser.end_sequence(),
            },
        }
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
            Compound::Map { ser, .. } | Compound::Seq { ser, .. } => {
                ser.in_key = true;
                key.serialize(&mut **ser)?;
                ser.in_key = false;
            }
        }

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Compound::Map { ser, .. } | Compound::Seq { ser, .. } => {
                ser.begin_object_value()?;
                value.serialize(&mut **ser)?;
                ser.end_object_value()
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Compound::Map { ser, state, .. } | Compound::Seq { ser, state } => match state {
                CompoundState::Empty => Ok(()),
                _ => ser.end_object_value(),
            },
        }
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
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;
    use serde::Serialize;

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

    #[test]
    fn test_unit_struct() {
        #[derive(Serialize)]
        struct Example;

        let formatter = PrettyFormatterConfig::pretty();
        let result = to_pretty_string(&Example, formatter);
        assert_eq!(result, Ok("{}".to_string()));
    }

    #[test]
    fn test_serialize_newtype_struct() {
        #[derive(Serialize)]
        struct Measurement(u8);

        let formatter = PrettyFormatterConfig::pretty();
        let result = to_pretty_string(&Measurement(0), formatter);
        assert_eq!(result, Ok("0".to_string()));
    }

    const COMPLEX_KEY_EXPECTED: &str = r#"
? - 1
  - 2
: 34"#;

    #[test]
    fn test_serialize_complex_key() {
        let key = vec![1, 2];
        let value = 34;
        let map = BTreeMap::from([(key, value)]);

        let result = to_pretty_string(&map, PrettyFormatterConfig::pretty());
        assert_eq!(result, Ok(COMPLEX_KEY_EXPECTED.to_string()));
    }
}
