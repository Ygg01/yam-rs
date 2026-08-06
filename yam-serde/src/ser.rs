use crate::binary;
use crate::escape_str::{CanBeScalar, escape_double_quotes, escape_single_quotes};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Error, Write};
use ser::{SerializeSeq, Serializer};
use serde_core::ser::{SerializeMap, SerializeStructVariant};
use serde_core::{Serialize, ser};
use unicode_segmentation::UnicodeSegmentation;
use yam_core::prelude::ScalarType;

trait YamlWhitespace {
    fn is_splittable_ws(&self) -> bool;
    fn is_last_char_splittable_ws(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FlowStyle {
    Plain,
    DoubleQuote,
    SingleQuote,
}

impl FlowStyle {
    pub(crate) fn to_scalar_type(self) -> ScalarType {
        match self {
            FlowStyle::Plain => ScalarType::Plain,
            FlowStyle::DoubleQuote => ScalarType::DoubleQuote,
            FlowStyle::SingleQuote => ScalarType::SingleQuote,
        }
    }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
enum SerializerState {
    #[default]
    Root,
    Block,
    Flow,
    ExplicitKey,
    ExplicitValue,
    FlowKey,
    BlockKey,
}

impl SerializerState {
    #[inline]
    pub(crate) fn switch_to_base(&self) {
        todo!()
    }
    #[inline]
    fn is_block_form(&self) -> bool {
        matches!(
            self,
            SerializerState::Block | SerializerState::ExplicitKey | SerializerState::BlockKey
        )
    }

    #[inline]
    fn is_key(&self) -> bool {
        matches!(
            self,
            SerializerState::FlowKey | SerializerState::BlockKey | SerializerState::ExplicitKey
        )
    }

    #[inline]
    fn is_flow_restricted(&self) -> bool {
        matches!(
            self,
            SerializerState::FlowKey | SerializerState::BlockKey | SerializerState::ExplicitKey
        )
    }

    #[inline]
    fn is_explicit_map(&self) -> bool {
        matches!(
            self,
            SerializerState::ExplicitKey | SerializerState::ExplicitValue
        )
    }
}

#[derive(Debug, Default)]
pub struct YamSerializer<W> {
    /// This string starts empty and JSON is appended as values are serialized.
    pub(crate) writer: W,
    pub(crate) indent_pos: u32,
    pub(crate) current_depth: u32,
    /// Pretty configuration option for formatting
    pub(crate) formatter: PrettyFormatterConfig,
    pub(crate) indentor_len: u32,
    complex_key_prefix: String,
    serializer_state: SerializerState,
    key_value_separator: String,
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
            indent_pos: 0,
            current_depth: 1,
            indentor_len: indentor_size,
            complex_key_prefix: String::new(),
            serializer_state: SerializerState::Root,
            key_value_separator: String::new(),
        }
    }

    #[inline]
    pub(crate) fn use_block_form(&mut self) -> bool {
        let switch_to_flow = self.current_depth > self.formatter.block_depth_limit;
        if switch_to_flow && self.serializer_state.is_block_form() {
            self.serializer_state = SerializerState::Flow;
        } else if !switch_to_flow && !self.serializer_state.is_block_form() {
            self.serializer_state = SerializerState::Block;
        }

        self.serializer_state.is_block_form()
    }

    pub(crate) fn begin_object(&mut self) -> Result<(), Error> {
        self.current_depth += 1;
        if !self.use_block_form() {
            self.write_ascii("{")?;
        }
        Ok(())
    }

    fn preferred_string(&self, string: &str) -> ScalarType {
        let in_block_form = self.serializer_state.is_block_form();
        let in_flow_restricted = self.serializer_state.is_flow_restricted();
        // Are we in key or another context?
        let mut preferred_style = if self.serializer_state.is_key() {
            self.formatter.key_preferred_style
        } else if self.current_depth == 1 {
            // In root use root preferred style
            self.formatter.root_preferred_style
        } else if !in_block_form {
            self.formatter.flow_string_style.to_scalar_type()
        } else {
            // Otherwise usually it's block
            self.formatter.block_preferred_style
        };

        // Plain style is one style that can't serialize a given string
        if preferred_style == ScalarType::Plain && !string.can_be_plain(in_flow_restricted) {
            preferred_style = self.formatter.flow_string_style.to_scalar_type();
        }

        preferred_style
    }

    #[inline]
    pub(crate) fn end_object(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
            self.write_ascii("}")?;
        }
        self.current_depth -= 1;
        Ok(())
    }

    #[inline]
    pub(crate) fn begin_sequence(&mut self) -> Result<(), Error> {
        if !self.use_block_form() {
            self.write_ascii("{")?;
        } else {
            self.write_prefix(self.complex_key_prefix.clone())?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn begin_sequence_value(&mut self, info: &CompoundInfo) -> Result<(), Error> {
        let first = info.is_first();
        self.write_to_indent(info.indent)?;
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
        if self.use_block_form() {
            self.complex_key_prefix.push_str("? ");
        }
        if !is_first && !self.serializer_state.is_block_form() {
            self.write_ascii(",")?;
        } else if !is_first && self.serializer_state.is_block_form() {
            self.write_nl()?;
        }
        self.serializer_state = SerializerState::BlockKey;
        Ok(())
    }

    #[inline]
    pub(crate) fn end_object_key(&mut self) -> Result<(), Error> {
        self.complex_key_prefix.clear();
        if !self.serializer_state.is_explicit_map() {
            self.key_value_separator = ": ".to_string();
        }
        Ok(())
    }

    pub(crate) fn begin_object_value(&mut self, is_collection: bool) -> Result<(), Error> {
        if self.serializer_state.is_explicit_map() {
            self.write_indent(self.current_depth.saturating_sub(1))?;
            self.complex_key_prefix = "".to_string();
        } else if is_collection {
            self.write_ascii(":")?;
            self.write_indent(self.current_depth)?;
            self.key_value_separator = "".to_string();
        }
        self.serializer_state = SerializerState::Block;
        self.write_separator()?;

        Ok(())
    }

    pub(crate) fn end_object_value(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn write_seq_start(&mut self) -> Result<(), Error> {
        if self.use_block_form() {
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
        self.indent_pos += 1;
        res
    }

    fn write_string(&mut self, str: &str) -> Result<(), Error> {
        let res = self.writer.write_str(str);
        let str_count: u32 = str
            .graphemes(true)
            .count()
            .try_into()
            .expect("Expected less than u32::MAX sized line");
        self.indent_pos += str_count;
        res
    }

    #[inline]
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
        self.indent_pos += str.len() as u32;
        res
    }

    fn write_separator(&mut self) -> Result<(), Error> {
        let res = self.writer.write_str(&self.key_value_separator);
        self.indent_pos += self.key_value_separator.len() as u32;
        res
    }

    #[inline]
    fn write_nl(&mut self) -> Result<(), Error> {
        let res = self.writer.write_char('\n');
        self.indent_pos = 0;
        res
    }

    #[inline]
    fn write_prefix(&mut self, prefix: String) -> Result<(), Error> {
        self.write_ascii(prefix.as_str())?;
        Ok(())
    }

    fn write_to_indent(&mut self, indent: u32) -> Result<(), Error> {
        if indent > self.indent_pos {
            let diff = (indent - self.indent_pos) as usize;
            let indent = " ".repeat(diff);
            self.writer.write_str(&indent)?;
        }
        Ok(())
    }

    fn is_time_to_split(&self, buff_len: u32) -> bool {
        self.indent_pos + buff_len > self.formatter.pref_string_length
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
        let corrected_indent = indent.saturating_sub(1);
        for _ in 0..corrected_indent {
            self.writer.write_str(&self.formatter.indentor)?;
        }
        self.indent_pos = corrected_indent * self.indentor_len;
        Ok(())
    }

    fn write_single_line(&mut self, fence: &str, escaped_str: &str) -> Result<(), Error> {
        self.write_string(fence)?;
        self.write_string(escaped_str)?;
        self.write_string(fence)?;
        let grapheme_count: u32 = escaped_str.graphemes(true).count().try_into().unwrap();
        self.indent_pos += 2 * (fence.len() as u32) + grapheme_count;
        Ok(())
    }

    fn write_block_string(&mut self, is_folded: bool, str: &str) -> Result<(), Error> {
        let mut string_writer = String::with_capacity(str.len() * 2);
        let chr = if is_folded { '>' } else { '|' };
        string_writer.push_str(str);

        // Write the pipe without updating position
        self.writer.write_char(chr)?;
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
        self.indent_pos = line_buff_grapheme_len;
        self.write_ascii(suffix)?;
        Ok(())
    }
}

impl<W> YamSerializer<W> {
    #[inline]
    pub fn new_simple(writer: W) -> Self {
        YamSerializer {
            writer,
            formatter: PrettyFormatterConfig::default(),
            indent_pos: 0,
            indentor_len: 0,
            current_depth: 0,
            complex_key_prefix: String::with_capacity(2),
            serializer_state: Default::default(),
            key_value_separator: String::with_capacity(2),
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

    /// How to format a string in block style
    pub block_preferred_style: ScalarType,

    /// How to format a string in root
    pub root_preferred_style: ScalarType,

    /// How to format a string in block style
    pub flow_string_style: FlowStyle,

    pub key_preferred_style: ScalarType,

    /// Whether to prefer string to fit in a single line
    pub compat_strings: bool,
}

impl Default for PrettyFormatterConfig {
    fn default() -> Self {
        Self {
            block_depth_limit: 0,
            pref_string_length: 80,
            indentor: Cow::Borrowed(""),
            new_line: Cow::Borrowed(""),
            null_format: Cow::Borrowed(""),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compat_strings: false,
        }
    }
}

impl PrettyFormatterConfig {
    pub fn pretty() -> Self {
        Self {
            block_depth_limit: 10,
            pref_string_length: 80,
            indentor: Cow::Borrowed("  "),
            new_line: Cow::Borrowed("\n"),
            null_format: Cow::Borrowed("null"),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compat_strings: false,
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
        self.indent_pos += str.len() as u32;
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
        let prefer_single_line = self.formatter.compat_strings;
        match self.preferred_string(v) {
            ScalarType::Plain => {
                if prefer_single_line {
                    self.write_single_line("", v)
                } else {
                    self.write_multi_line_string("", v, "")
                }
            }
            ScalarType::Folded => self.write_block_string(true, v),
            ScalarType::Literal => self.write_block_string(true, v),
            ScalarType::SingleQuote => {
                let mut var = String::with_capacity(v.len() * 2);
                escape_single_quotes(&mut var, v)?;
                if prefer_single_line {
                    self.write_single_line("'", &var)?;
                } else {
                    self.write_multi_line_string("'", &var, "'")?;
                }
                Ok(())
            }
            ScalarType::DoubleQuote => {
                let mut var = String::with_capacity(v.len() * 2);
                escape_double_quotes(&mut var, v)?;
                if prefer_single_line {
                    self.write_single_line("\"", &var)?;
                } else {
                    self.write_multi_line_string("\"", &var, "\"")?;
                }
                Ok(())
            }
        }
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
                    .saturating_sub(self.indent_pos);
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
        self.indent_pos += self.formatter.null_format.len() as u32;
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

        self.begin_object_value(false)?;
        value.serialize(&mut *self)?;
        self.end_object_value()?;

        self.end_object()?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.begin_sequence()?;
        if len == Some(0) {
            self.end_sequence()?;
            Ok(Compound::empty(self))
        } else {
            Ok(Compound::first(self))
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
        self.begin_object_value(false)?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.begin_object()?;
        let indent = self.indent_pos;
        if len == Some(0) {
            self.end_object()?;
            Ok(Compound::empty(self))
        } else {
            Ok(Compound::first(self))
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
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.begin_object()?;

        self.begin_object_key(true)?;
        self.serialize_str(variant)?;
        self.end_object_key()?;

        self.begin_object_value(false)?;
        self.serialize_map(Some(len))
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
#[derive(Eq, PartialEq)]
pub struct CompoundInfo {
    state: CompoundState,
    prev_state: SerializerState,
    current_is_collection: bool,
    indent: u32,
    seq: bool,
}

impl CompoundInfo {
    fn is_first(&self) -> bool {
        matches!(self.state, CompoundState::First)
    }
}

#[doc(hidden)]
pub struct Compound<'a, W: Write> {
    ser: &'a mut YamSerializer<W>,
    info: CompoundInfo,
}

impl<'a, W: Write> Compound<'a, W> {
    pub(crate) fn empty(ser: &'a mut YamSerializer<W>) -> Self {
        let prev_state = ser.serializer_state.clone();
        Compound {
            ser,
            info: CompoundInfo {
                state: CompoundState::Empty,
                prev_state,
                current_is_collection: false,
                indent: 0,
                seq: false,
            },
        }
    }

    pub(crate) fn first(ser: &'a mut YamSerializer<W>) -> Self {
        let prev_state = ser.serializer_state.clone();
        Compound {
            ser,
            info: CompoundInfo {
                state: CompoundState::First,
                prev_state,
                current_is_collection: false,
                indent: 0,
                seq: false,
            },
        }
    }
    #[inline]
    pub(crate) fn set_current_is_collection(&mut self, is_collection: bool) {
        self.info.current_is_collection = is_collection;
    }
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
        self.ser.begin_sequence_value(&self.info)?;
        self.info.state = CompoundState::Rest;
        value.serialize(&mut *self.ser)?;
        self.ser.end_sequence_value()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.info.state {
            CompoundState::Empty => Ok(()),
            _ => self.ser.end_sequence(),
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
        self.ser.begin_object_key(self.info.is_first())?;
        key.serialize(&mut *self.ser)?;
        self.ser.end_object_key()?;

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.ser.begin_object_value(false)?;
        value.serialize(&mut *self.ser)?;
        self.info.state = CompoundState::Rest;
        self.ser.end_object_value()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.info.state {
            CompoundState::Empty => Ok(()),
            _ => self.ser.end_object_value(),
        }
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: Write,
{
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

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: Write,
{
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

impl<'a, W> SerializeStructVariant for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.set_current_is_collection(true);
        self.serialize_key(key)?;
        self.serialize_value(value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
where
    W: Write,
{
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

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
