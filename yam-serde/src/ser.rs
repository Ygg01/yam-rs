use crate::escape_str::{CanBeScalar, escape_double_quotes, escape_single_quotes};
use crate::{PrettyFormatterConfig, binary};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Error, Write};
use core::num::NonZero;
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

pub type NonZeroKVSeparator = NonZero<CompoundStyle>;

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
    Block,
    Flow,
    ExplicitKey,
    ExplicitValue,
    FlowKey,
    FlowValue,
    BlockValue,
    BlockKey,
}

impl SerializerState {
    fn to_compound_style(self) -> CompoundStyle {
        match self {
            SerializerState::Block | SerializerState::BlockKey | SerializerState::BlockValue => {
                CompoundStyle::Block
            }
            SerializerState::Flow | SerializerState::FlowKey | SerializerState::FlowValue => {
                CompoundStyle::Flow
            }
            SerializerState::ExplicitKey | SerializerState::ExplicitValue => {
                CompoundStyle::ExplicitMap
            }
        }
    }

    #[inline]
    fn is_block_form(&self) -> bool {
        matches!(
            self,
            SerializerState::BlockValue
                | SerializerState::ExplicitKey
                | SerializerState::BlockKey
                | SerializerState::ExplicitValue
                | SerializerState::Block
        )
    }

    pub(crate) fn is_in_map(&self) -> bool {
        matches!(
            self,
            SerializerState::BlockValue | SerializerState::ExplicitValue
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
    indent_pos: u32,
    current_depth: u32,
    /// Pretty configuration option for formatting
    formatter: PrettyFormatterConfig,
    indentor_len: u32,
    /// Serialization states
    serializer_state: SerializerState,
    is_scalar: bool,
    key_val_sep: Option<CompoundStyle>,
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
            is_scalar: false,
            serializer_state: SerializerState::Block,
            key_val_sep: Default::default(),
        }
    }

    pub(crate) fn flush_block_value(&mut self) -> Result<(), Error> {
        match self.key_val_sep.take() {
            Some(CompoundStyle::Block) if self.is_scalar => {
                self.is_scalar = false;
                self.write_nspaces(self.indentor_len.saturating_sub(1))?;
            }
            Some(CompoundStyle::Block) => {
                self.write_indent(self.current_depth.saturating_sub(1))?;
            }
            _ => {}
        };

        Ok(())
    }

    pub(crate) fn finish(&self) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_nums<T: Display>(&mut self, value: T) -> Result<(), Error> {
        write!(self.writer, "{value}")?;
        Ok(())
    }

    #[inline]
    pub(crate) fn use_block_form(&mut self) -> bool {
        let switch_to_flow = self.current_depth > self.formatter.block_depth_limit;
        let is_block_value = matches!(
            self.serializer_state,
            SerializerState::BlockValue | SerializerState::ExplicitValue
        );

        if switch_to_flow && is_block_value {
            self.serializer_state = SerializerState::FlowValue;
        }

        self.serializer_state.is_block_form()
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

    fn get_collection_serializer(&mut self) -> Compound<'_, W>
    where
        W: Write,
    {
        match self.serializer_state {
            SerializerState::Block => Compound::new(self, CompoundStyle::Block, None, true),
            SerializerState::Flow => Compound::new(self, CompoundStyle::Flow, None, true),
            SerializerState::ExplicitKey | SerializerState::ExplicitValue => {
                Compound::new(self, CompoundStyle::ExplicitMap, None, true)
            }
            _ => todo!(),
        }
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

    #[inline]
    fn write_block_seq_start(&mut self) -> Result<(), Error> {
        let mut string = String::with_capacity(self.indentor_len as usize);
        string.push('-');
        string.write_str(&" ".repeat((self.indentor_len as usize).saturating_sub(1)))?;
        self.writer.write_str(&string)?;
        self.indent_pos += string.len() as u32;
        Ok(())
    }

    #[inline]
    fn write_explicit_map_start(&mut self) -> Result<(), Error> {
        let mut string = String::with_capacity(self.indentor_len as usize);
        string.push('?');
        string.write_str(&" ".repeat((self.indentor_len as usize).saturating_sub(1)))?;
        self.writer.write_str(&string)?;
        self.indent_pos += string.len() as u32;
        Ok(())
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

    fn write_nspaces(&mut self, pos: u32) -> Result<(), Error> {
        let indent = " ".repeat(pos as usize);
        self.writer.write_str(&indent)?;
        self.indent_pos += pos;

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
        let corrected_indent =
            push_indent_to_writer(&mut self.writer, indent, &self.formatter.indentor)?;
        self.indent_pos = corrected_indent * self.indentor_len;
        Ok(())
    }

    fn write_indentor(&mut self, repeat: u32) -> Result<(), Error> {
        let corrected_indent = self.formatter.indentor.repeat(repeat as usize);
        self.writer.write_str(&corrected_indent)?;
        self.indent_pos += corrected_indent.len() as u32;

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
            is_scalar: false,
            serializer_state: Default::default(),
            key_val_sep: Default::default(),
        }
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
        self.is_scalar = true;
        self.flush_block_value()?;
        let str = if v { "true" } else { "false" };
        self.writer.write_str(str)?;
        self.indent_pos += str.len() as u32;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.serialize_nums(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        if v.is_nan() && v.is_sign_negative() {
            write!(self.writer, "-")?;
        }

        write!(self.writer, "{v}")?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        if v.is_nan() && v.is_sign_negative() {
            write!(self.writer, "-")?;
        }

        write!(self.writer, "{v}")?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        let string_chr = if v == '\'' { '"' } else { '\'' };

        self.writer.write_char(string_chr)?;
        self.writer.write_char(v)?;
        self.writer.write_char(string_chr)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
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
        self.is_scalar = true;
        self.flush_block_value()?;
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
        self.is_scalar = true;
        self.flush_block_value()?;
        self.writer.write_str(&self.formatter.null_format)?;
        self.indent_pos += self.formatter.null_format.len() as u32;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.write_ascii("{}")?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_ascii("{ ")?;
        self.write_string(variant)?;
        self.write_ascii(" }")?;
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
        let mut collection_serializer = Compound::new(
            self,
            self.serializer_state.to_compound_style(),
            Some(1),
            false,
        );

        collection_serializer.begin_object()?;

        collection_serializer.serialize_key(variant)?;

        collection_serializer.serialize_value(value)?;

        collection_serializer.end_object()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let mut collection_serializer =
            Compound::new(self, self.serializer_state.to_compound_style(), len, true);
        collection_serializer.begin_sequence()?;

        Ok(collection_serializer)
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
        let mut struct_serializer = Compound::new(
            self,
            self.serializer_state.to_compound_style(),
            Some(len),
            true,
        );

        struct_serializer.begin_object()?;
        struct_serializer.serialize_key(name)?;

        Ok(struct_serializer)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let mut collection_serializer =
            Compound::new(self, self.serializer_state.to_compound_style(), len, true);

        collection_serializer.begin_object()?;
        Ok(collection_serializer)
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
        let mut collection_serializer = Compound::new(self, CompoundStyle::Flow, Some(len), true);
        collection_serializer.begin_object()?;

        collection_serializer.serialize_key(variant)?;

        Ok(collection_serializer)
    }
}

#[doc(hidden)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct CompoundInfo {
    state: CompoundState,
    is_root: bool,
    depth: u32,
}

#[doc(hidden)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum CompoundState {
    Empty,
    First,
    Rest,
}

impl CompoundInfo {
    fn is_first(&self) -> bool {
        matches!(self.state, CompoundState::First)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
#[repr(u8)]
pub enum CompoundStyle {
    #[default]
    Block = 1,
    Flow = 2,
    ExplicitMap = 3,
}

#[allow(private_interfaces)]
#[doc(hidden)]
pub enum Compound<'a, W: Write> {
    Flow(FlowCollection<'a, W>),
    Block(BlockCollection<'a, W>),
    ExplicitMap(ExplicitMapCollection<'a, W>),
}

impl<'a, W> Compound<'a, W>
where
    W: Write,
{
    fn get_style(&self) -> CompoundStyle {
        match self {
            Compound::Flow(_) => CompoundStyle::Flow,
            Compound::Block(_) => CompoundStyle::Block,
            Compound::ExplicitMap(_) => CompoundStyle::ExplicitMap,
        }
    }

    fn to_style(self, style: CompoundStyle) -> Self {
        if self.get_style() == style {
            return self;
        }

        let (ser, info) = match self {
            Compound::Flow(flow) => (flow.ser, flow.info),
            Compound::Block(block) => (block.ser, block.info),
            Compound::ExplicitMap(explicit_map) => (explicit_map.ser, explicit_map.info),
        };

        match style {
            CompoundStyle::Flow => Compound::Flow(FlowCollection { ser, info }),
            CompoundStyle::Block => Compound::Block(BlockCollection { ser, info }),
            CompoundStyle::ExplicitMap => {
                Compound::ExplicitMap(ExplicitMapCollection { ser, info })
            }
        }
    }
}

#[doc(hidden)]
pub(crate) struct FlowCollection<'a, W: Write> {
    ser: &'a mut YamSerializer<W>,
    info: CompoundInfo,
}

#[doc(hidden)]
pub(crate) struct BlockCollection<'a, W: Write> {
    ser: &'a mut YamSerializer<W>,
    info: CompoundInfo,
}
#[doc(hidden)]
pub(crate) struct ExplicitMapCollection<'a, W: Write> {
    ser: &'a mut YamSerializer<W>,
    info: CompoundInfo,
}

trait YamlSerializerTrait {
    fn begin_obj(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn begin_obj_key(&mut self) -> Result<(), Error>;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize;

    fn begin_obj_val(&mut self) -> Result<(), Error>;

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize;

    fn end_obj(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn begin_seq(&mut self) -> Result<(), Error>;

    fn serialize_seq_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize;

    fn end_seq(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

// impl<'a, W: Write> YamlSerializerTrait for FlowCollection<'a, W> {
//     fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize,
//     {
//         key.serialize(&mut *self.ser)?;
//         Ok(())
//     }
//
//     fn begin_obj_val(&mut self) -> Result<(), Error> {
//         self.ser.write_ascii(": ")?;
//         Ok(())
//     }
//
//     fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize,
//     {
//         if !self.info.is_first() {
//             self.ser.write_ascii(", ")?;
//         }
//         value.serialize(&mut *self.ser)?;
//         Ok(())
//     }
//
//     fn serialize_seq_element<T>(&mut self, _value: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!()
//     }
// }
impl<'a, W: Write> YamlSerializerTrait for BlockCollection<'a, W> {
    fn begin_obj_key(&mut self) -> Result<(), Error> {
        self.ser.write_nl()
    }

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.ser)
    }

    fn begin_obj_val(&mut self) -> Result<(), Error> {
        self.ser.key_val_sep = Some(CompoundStyle::Block);
        self.ser.write_ascii(":")?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn begin_seq(&mut self) -> Result<(), Error> {
        self.ser.current_depth += 1;
        self.ser.flush_block_value()?;
        self.ser.write_block_seq_start()?;

        Ok(())
    }

    fn serialize_seq_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if self.info.is_first() {
            self.info.state = CompoundState::Rest;
        } else {
            let indent_diff = self.ser.current_depth.saturating_sub(self.info.depth);
            self.ser.write_indentor(indent_diff)?;
            self.ser.write_block_seq_start()?;
        }

        value.serialize(&mut *self.ser)?;
        if self.ser.is_scalar {
            self.ser.is_scalar = false;
            self.ser.write_nl()?;
        }
        Ok(())
    }

    fn end_seq(&mut self) -> Result<(), Error> {
        self.ser.current_depth -= 1;
        Ok(())
    }
}

// impl<'a, W> YamlSerializerTrait for ExplicitMapCollection<'a, W>
// where W: Write
// {
//     fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize
//     {
//         self.ser.write_explicit_map_start()?;
//         Ok(())
//     }
//
//     fn begin_obj_val(&mut self) -> Result<(), Error> {
//         todo!()
//     }
//
//     fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize
//     {
//         todo!()
//     }
//
//     fn serialize_seq_element<T>(&mut self, value: &T) -> Result<(), Error>
//     where
//         T: ?Sized + Serialize
//     {
//         todo!()
//     }
// }

impl<'a, W> Compound<'a, W>
where
    W: Write,
{
    fn new(
        ser: &'a mut YamSerializer<W>,
        style: CompoundStyle,
        len: Option<usize>,
        is_root: bool,
    ) -> Self {
        let state = if len == Some(0) {
            CompoundState::Empty
        } else {
            CompoundState::First
        };
        let info = CompoundInfo {
            state,
            is_root,
            depth: 2,
        };
        match style {
            CompoundStyle::Flow => Compound::Flow(FlowCollection { ser, info }),
            CompoundStyle::Block => Compound::Block(BlockCollection { ser, info }),
            CompoundStyle::ExplicitMap => {
                Compound::ExplicitMap(ExplicitMapCollection { ser, info })
            }
        }
    }

    pub(crate) fn begin_object(&mut self) -> Result<(), Error> {
        self.ser().current_depth += 1;
        Ok(())
    }

    pub(crate) fn end_object(&mut self) -> Result<(), Error> {
        self.ser().current_depth += 1;
        Ok(())
    }

    pub(crate) fn begin_sequence(&mut self) -> Result<(), Error> {
        match self {
            Compound::Block(bc) => bc.begin_seq(),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    pub(crate) fn sequence_elem<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Compound::Block(bc) => bc.serialize_seq_element(value),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    pub(crate) fn end_sequence(&mut self) -> Result<(), Error> {
        match self {
            Compound::Block(bc) => bc.end_seq(),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    #[inline]
    pub(crate) fn begin_object_key(&mut self) -> Result<(), Error> {
        match self {
            Compound::Block(bc) => bc.begin_obj_key(),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    #[inline]
    pub(crate) fn begin_object_value(&mut self) -> Result<(), Error> {
        match self {
            Compound::Block(bc) => bc.begin_obj_val(),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    fn ser(&mut self) -> &mut YamSerializer<W> {
        match self {
            Compound::Flow(FlowCollection { ser, .. })
            | Compound::Block(BlockCollection { ser, .. })
            | Compound::ExplicitMap(ExplicitMapCollection { ser, .. }) => ser,
        }
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
        match self {
            Compound::Block(bc) => bc.serialize_seq_element(value),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            // Compound::Flow(mut fc) => fc.end_seq(),
            Compound::Block(mut fc) => fc.end_seq(),
            Compound::ExplicitMap(_) | Compound::Flow(_) => todo!(),
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
        self.begin_object_key()?;
        match self {
            // Compound::Flow(fc) => {
            //     fc.serialize_key(key)?;
            // }
            Compound::Block(bc) => {
                bc.serialize_key(key)?;
            }
            _ => todo!(),
        };
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.begin_object_value()?;
        match self {
            // Compound::Flow(fc) => fc.serialize_value(value),
            Compound::Block(bc) => bc.serialize_value(value),
            _ => todo!(),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            // Compound::Flow(mut fc) => fc.end_obj(),
            Compound::Block(mut bc) => bc.end_obj(),
            _ => todo!(),
        }
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.sequence_elem(value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_sequence()
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
        self.sequence_elem(_value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_sequence()
    }
}

impl<'a, W> SerializeStructVariant for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.begin_object()?;
        self.serialize_key(_key)?;
        self.serialize_value(_value)?;
        self.end_object()
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
        self.serialize_element(_value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
        self.begin_object()?;

        self.serialize_key(key)?;

        self.serialize_value(value)?;
        self.end_object()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[doc(hidden)]
/// Writes a specified level of indentation to a given writer using a specified indentor string.
///
/// # Parameters
/// - `writer`: A mutable reference to an object implementing the `Write` trait,
///   where the indentation will be written.
/// - `indent`: The desired number of indentation levels. If this value is 1, no
///   indentation will be written.
/// - `indentor`: The string used to represent a single level of indentation,
///   such as `"    "` or `"\t"`.
///
/// # Returns
/// - `Result<u32, Error>`: Returns the number of bytes written
///   (adjusted by subtracting 1 from the requested `indent` value) wrapped in `Ok`, or an error
///   wrapped in `Err` if the write operation fails.
///
/// # Errors
/// - Returns an error if writing to the `writer` fails, encapsulated in the `Error` type.
///
/// # Notes
/// - This function ensures that the computed indentation level (`indent.saturating_sub(1)`) is
///   non-negative, even if `indent` is 0.
/// - The function writes the `indentor` string `corrected_indent` times to the given `writer`.
///
pub fn push_indent_to_writer<W: Write>(
    writer: &mut W,
    indent: u32,
    indentor: &str,
) -> Result<u32, Error> {
    let corrected_indent = indent.saturating_sub(1);
    for _ in 0..corrected_indent {
        writer.write_str(indentor)?;
    }
    Ok(corrected_indent)
}
