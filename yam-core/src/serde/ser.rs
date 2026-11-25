use crate::binary;
use crate::emitter::YamlWriter;
use crate::prelude::ScalarType;
use crate::serde::YamlFormatter;
use crate::serde::escape_str::{CanBeScalar, escape_double_quotes, escape_single_quotes};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::cmp::min;
use core::fmt::{Debug, Display, Error, Write};
use core::option::Option;
use ser::{SerializeSeq, Serializer};
use serde_core::ser::{SerializeMap, SerializeStructVariant};
pub(crate) use serde_core::{Serialize, ser};
use unicode_segmentation::UnicodeSegmentation;

trait YamlWhitespace {
    fn is_splittable_ws(&self) -> bool;
    fn is_last_char_splittable_ws(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FlowStyle {
    /// Unquoted flow strings e.g. `[foo, bar]`
    Plain,
    /// Double-quoted flow strings e.g. `["foo", "bar"]`. Default for JSON.
    DoubleQuote,
    /// Single-quoted flow strings e.g. `['foo', 'bar']`.
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
        self.bytes().last().is_some_and(|c| c == b' ' || c == b'\n')
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
    fn to_compound_style(self) -> YamlStyle {
        match self {
            SerializerState::Block | SerializerState::BlockKey | SerializerState::BlockValue => {
                YamlStyle::Block
            }
            SerializerState::Flow | SerializerState::FlowKey | SerializerState::FlowValue => {
                YamlStyle::Flow
            }
            SerializerState::ExplicitKey | SerializerState::ExplicitValue => YamlStyle::Explicit,
        }
    }

    #[inline]
    fn is_block_form(self) -> bool {
        matches!(
            self,
            SerializerState::BlockValue
                | SerializerState::ExplicitKey
                | SerializerState::BlockKey
                | SerializerState::ExplicitValue
                | SerializerState::Block
        )
    }

    #[inline]
    fn is_key(self) -> bool {
        matches!(
            self,
            SerializerState::FlowKey | SerializerState::BlockKey | SerializerState::ExplicitKey
        )
    }

    #[inline]
    fn is_flow_restricted(self) -> bool {
        matches!(
            self,
            SerializerState::FlowKey | SerializerState::BlockKey | SerializerState::ExplicitKey
        )
    }
}

pub struct YamSerializer<W> {
    /// This string starts empty and JSON is appended as values are serialized.
    pub(crate) writer: YamlWriter<W>,
    /// Pretty configuration option for formatting
    formatter: YamlFormatter,
    /// Serialization states
    serializer_states: Vec<SerializerState>,
    is_scalar: bool,
    key_val_sep: Option<YamlStyle>,
}

impl<W> YamSerializer<W>
where
    W: Write,
{
    pub(crate) fn ensure_indent(&mut self) -> Result<(), Error> {
        let expected_indent = self.writer.indentor_len() * (self.current_depth() - 1);
        if self.writer.indent_pos() > expected_indent {
            self.writer.writeln()?;
        }
        let diff_indent = expected_indent - self.writer.indent_pos();
        self.writer.write_n_spaces(diff_indent)
    }

    pub(crate) fn flush_block_value(&mut self) -> Result<(), Error> {
        match self.key_val_sep.take() {
            Some(YamlStyle::Block) if self.is_scalar => {
                self.is_scalar = false;
                let spaces = min(
                    (self.writer.indentor_len() * self.current_depth()).saturating_sub(1),
                    1,
                );
                self.writer.write_ascii(":")?;
                self.writer.write_n_spaces(spaces)?;
            }
            Some(YamlStyle::Block) => {
                self.writer.write_ascii(":")?;
                self.writer.write_newline_indent(self.current_depth())?;
            }
            Some(YamlStyle::Explicit) => {
                self.ensure_indent()?;
                self.writer.write_ascii(": ")?;
            }
            Some(YamlStyle::Flow) => {
                self.writer.write_ascii(": ")?;
            }
            _ => {}
        }

        Ok(())
    }

    fn serialize_nums<T: Display>(&mut self, value: T) -> Result<(), Error> {
        self.writer.write_ascii(&value.to_string())
    }

    fn preferred_string(&self, string: &str) -> ScalarType {
        let in_block_form = self.serializer_state().is_block_form();
        let in_flow_restricted = self.serializer_state().is_flow_restricted();
        // Are we in key or another context?
        let mut preferred_style = if self.serializer_state().is_key() {
            self.formatter.key_preferred_style
        } else if self.current_depth() == 0 {
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
            preferred_style = match self.formatter.flow_string_style {
                FlowStyle::SingleQuote => ScalarType::SingleQuote,
                FlowStyle::Plain | FlowStyle::DoubleQuote => ScalarType::DoubleQuote,
            }
        }

        preferred_style
    }

    #[inline]
    fn write_before_block_elem(&mut self, info: &mut CompoundInfo) -> Result<(), Error> {
        if !info.is_first() {
            // write indentation
            let diff_depth = self.current_depth().saturating_sub(info.depth as usize)
                * self.writer.indentor_len();
            self.writer.write_n_spaces(diff_depth)?;
            self.writer.write_block_seq_start('-')?;
        }

        info.state = CompoundState::Rest;
        Ok(())
    }

    #[inline]
    fn write_after_block_elem(&mut self) -> Result<(), Error> {
        if self.is_scalar {
            self.is_scalar = false;
            self.writer.writeln()?;
        }
        Ok(())
    }

    #[inline]
    fn is_time_to_split(&self, buff_len: usize) -> bool {
        self.writer.indent_pos() + buff_len > self.formatter.pref_string_length as usize
    }

    fn write_flow_string_inline(&mut self, input: &str) -> Result<(), Error> {
        let mut string_writer = String::with_capacity(input.len() * 2);

        let style = if input.can_be_plain(true) {
            self.formatter.flow_string_style
        } else {
            FlowStyle::DoubleQuote
        };

        match style {
            FlowStyle::Plain => {
                string_writer.push_str(input);
            }
            FlowStyle::DoubleQuote => {
                string_writer.push('"');
                escape_double_quotes(&mut string_writer, input)?;
                string_writer.push('"');
            }
            FlowStyle::SingleQuote => {
                string_writer.push('\'');
                escape_single_quotes(&mut string_writer, input)?;
                string_writer.push('\'');
            }
        }
        self.writer.write_str(&string_writer)?;
        Ok(())
    }

    fn write_block_string(
        &mut self,
        is_folded: bool,
        str: &str,
        at_indent: usize,
    ) -> Result<(), Error> {
        let mut string_writer = String::with_capacity(str.len() * 2);

        let chr = if str.ends_with('\n') {
            let len_min_1 = str.len().saturating_sub(1);
            string_writer.push_str(&str[0..len_min_1]);
            if is_folded { ">" } else { "|" }
        } else {
            string_writer.push_str(str);
            if is_folded { ">-" } else { "|-" }
        };

        // Write the pipe without updating position
        self.writer.write_ascii(chr)?;
        self.writer.writeln()?;
        // then indent the string and write the block.
        self.writer.write_indent(at_indent)?;
        self.write_multi_line_string("", &string_writer, "")?;

        Ok(())
    }

    #[inline]
    fn line_split_at(&mut self, line_buff: &str, line_split: &str) -> Result<(), Error> {
        let escaped = if line_split == " " { "" } else { "\n" };
        self.writer.write_str(line_buff)?;
        self.writer.write_str(escaped)?;
        self.writer.write_newline_indent(self.current_depth())
    }

    #[inline]
    fn write_multi_line_string(
        &mut self,
        prefix: &str,
        str: &str,
        suffix: &str,
    ) -> Result<(), Error> {
        if self.is_time_to_split(0) {
            self.writer.write_newline_indent(self.current_depth())?;
        }
        self.writer.write_ascii(prefix)?;

        let mut line_buff =
            String::with_capacity((self.formatter.pref_string_length + 20).try_into().unwrap());
        let mut line_buff_grapheme_len = 0;
        let word_bounds = str
            .split_word_bound_indices()
            .map(|(_, word)| (word, word.graphemes(true).count()))
            .collect::<Vec<(&str, usize)>>();

        for (word, grapheme_len) in word_bounds {
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
                    line_buff_grapheme_len = front.len();
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
        self.writer.set_indent_pos(line_buff_grapheme_len);
        self.writer.write_ascii(suffix)?;
        Ok(())
    }
}

impl<W> YamSerializer<W> {
    #[inline]
    pub fn with_format(writer: W, formatter: YamlFormatter) -> Self {
        YamSerializer {
            writer: YamlWriter::new(writer),
            formatter,
            is_scalar: false,
            serializer_states: vec![],
            key_val_sep: Option::default(),
        }
    }

    #[inline]
    fn serializer_state(&self) -> SerializerState {
        match self.serializer_states.last() {
            Some(state) => *state,
            _ => self.formatter.root_style.into(),
        }
    }

    #[inline]
    fn serializer_state_mut(&mut self) -> &mut SerializerState {
        if self.serializer_states.is_empty() {
            self.serializer_states
                .push(self.formatter.root_style.into());
        }
        // Fine to use because above we ensure there is at least one style.
        self.serializer_states.last_mut().unwrap()
    }

    #[inline]
    pub(crate) fn current_depth(&self) -> usize {
        self.serializer_states.len()
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
        self.writer.write_bool(v)?;
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
        self.writer.write_ascii(&v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.writer.write_ascii(&v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.write_flow_string_inline(&v.to_string())?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        let prefer_single_line = self.formatter.compact_strings;
        match self.preferred_string(v) {
            ScalarType::Plain => {
                if prefer_single_line {
                    self.writer.write_inline(v, "")
                } else {
                    self.write_multi_line_string("", v, "")
                }
            }
            ScalarType::Folded => self.write_block_string(true, v, self.current_depth()),
            ScalarType::Literal => self.write_block_string(false, v, self.current_depth()),
            ScalarType::SingleQuote => {
                let mut var = String::with_capacity(v.len() * 2);
                escape_single_quotes(&mut var, v)?;
                if prefer_single_line {
                    self.writer.write_inline(&var, "'")?;
                } else {
                    self.write_multi_line_string("'", &var, "'")?;
                }
                Ok(())
            }
            ScalarType::DoubleQuote => {
                let mut var = String::with_capacity(v.len() * 2);
                escape_double_quotes(&mut var, v)?;
                if prefer_single_line {
                    self.writer.write_inline(&var, "\"")?;
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
        self.writer.write_ascii("!!binary ")?;
        let encoded_bytes = binary::encode_as_base64(v);
        if matches!(
            self.serializer_state(),
            SerializerState::FlowValue | SerializerState::Flow | SerializerState::FlowKey
        ) {
            self.writer.write_ascii("\"")?;
            self.writer.write_ascii(&encoded_bytes)?;
            self.writer.write_ascii("\"")?;
        } else {
            self.write_block_string(false, &encoded_bytes, self.current_depth() + 1)?;
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

    // Allowing this since we control null format
    #[allow(clippy::cast_possible_truncation)]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.writer.write_str(&self.formatter.null_format)?;
        self.writer
            .incr_indent_pos(self.formatter.null_format.len());
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.is_scalar = true;
        self.flush_block_value()?;
        self.writer.write_ascii("{}")?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.writer.write_ascii("{ ")?;
        self.write_flow_string_inline(variant)?;
        self.writer.write_ascii(" }")?;
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
        let mut collection_serializer = Compound::flow(self, Some(1));

        collection_serializer.begin_object()?;

        collection_serializer.serialize_key(variant)?;
        collection_serializer.serialize_value(value)?;

        collection_serializer.end_object()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let mut collection_serializer =
            Compound::new(self, self.serializer_state().to_compound_style(), len);
        collection_serializer.begin_seq()?;

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
        // Tuple struct (2, 4) is serialized as a `[2, 4]`
        let mut struct_serializer = Compound::flow(self, Some(len));
        struct_serializer.begin_seq()?;
        Ok(struct_serializer)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut struct_serializer = Compound::flow(self, Some(len));

        // Tuple variant is written as `Variant: ['values', 'in', 'tuple']
        struct_serializer.begin_object()?;

        // Serialize the `Variant` part
        struct_serializer.serialize_key(variant)?;

        // Serialize the `: ` part
        struct_serializer.begin_obj_val();

        // Reset the count for the sequence
        struct_serializer.info.reset_to_first();

        // Serialize the ['values', 'in', 'tuple']
        struct_serializer.begin_seq()?;

        Ok(struct_serializer)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let compound_style = self.serializer_states.last().copied().map_or(
            self.formatter.root_style,
            SerializerState::to_compound_style,
        );

        let mut collection_serializer = Compound::new(self, compound_style, len);

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
        let mut collection_serializer = Compound::flow(self, Some(len));

        // Struct variant is written as `Variant: { key: "value"}

        collection_serializer.begin_object()?;

        // Serialize the `Variant` part
        collection_serializer.serialize_key(variant)?;

        // Serialize the `: ` part
        collection_serializer.begin_obj_val();

        // Reset to first, for the rest of the serializer
        collection_serializer.info.reset_to_first();

        // Serialize the `{ key: "value"}` part

        Ok(collection_serializer)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&format!("{value}"))
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
    fn is_first(self) -> bool {
        matches!(self.state, CompoundState::First)
    }

    pub(crate) fn reset_to_first(&mut self) {
        self.state = CompoundState::First;
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
#[repr(u8)]
pub enum YamlStyle {
    #[default]
    Block = 1,
    Flow = 2,
    Explicit = 3,
}

#[allow(private_interfaces)]
#[doc(hidden)]
pub struct Compound<'a, W> {
    ser: &'a mut YamSerializer<W>,
    info: CompoundInfo,
    style: YamlStyle,
}

impl From<YamlStyle> for SerializerState {
    fn from(value: YamlStyle) -> Self {
        match value {
            YamlStyle::Block => SerializerState::Block,
            YamlStyle::Flow => SerializerState::Flow,
            YamlStyle::Explicit => SerializerState::ExplicitKey,
        }
    }
}

impl<'a, W> Compound<'a, W> {
    fn new(
        ser: &'a mut YamSerializer<W>,
        style: YamlStyle,
        len: Option<usize>,
        // is_root: bool,
    ) -> Self {
        let state = if len == Some(0) {
            CompoundState::Empty
        } else {
            CompoundState::First
        };
        let info = CompoundInfo {
            state,
            is_root: false,
            depth: 1,
        };
        Compound { ser, info, style }
    }

    fn flow(ser: &'a mut YamSerializer<W>, len: Option<usize>) -> Self {
        *ser.serializer_state_mut() = SerializerState::Flow;
        Compound::new(ser, YamlStyle::Flow, len)
    }
}

impl<W> Compound<'_, W>
where
    W: Write,
{
    #[inline]
    fn push_state_in_seq(&mut self) -> Result<(), Error> {
        let depth = u32::try_from(self.ser.serializer_states.len())
            .expect("Depth can't be greater than u32::MAX");

        if matches!(
            self.ser.serializer_state(),
            SerializerState::BlockKey | SerializerState::ExplicitKey
        ) && self.style == YamlStyle::Block
        {
            *self.ser.serializer_state_mut() = SerializerState::ExplicitValue;

            self.ser.writer.write_block_seq_start('?')?;
        }

        let state = if depth > self.ser.formatter.block_depth_limit
            || !self.ser.serializer_state().is_block_form()
        {
            SerializerState::Flow
        } else {
            SerializerState::Block
        };
        self.ser.serializer_states.push(state);

        Ok(())
    }

    #[inline]
    fn begin_seq(&mut self) -> Result<(), Error> {
        self.ser.is_scalar = false;
        self.push_state_in_seq()?;
        self.ser.flush_block_value()?;

        match self.style {
            YamlStyle::Block | YamlStyle::Explicit => {
                self.ser.writer.write_block_seq_start('-')?;
            }
            YamlStyle::Flow => {
                self.ser.writer.write_ascii("[")?;
                self.ser.writer.write_n_spaces(1)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn end_seq(&mut self) -> Result<(), Error> {
        if self.style == YamlStyle::Flow {
            self.ser.writer.write_n_spaces(1)?;
            self.ser.writer.write_ascii("]")?;
        }
        self.ser.serializer_states.pop();
        Ok(())
    }

    #[inline]
    fn begin_seq_elem(&'_ mut self) -> Result<(), Error> {
        match self.style {
            YamlStyle::Block | YamlStyle::Explicit => {
                self.ser.write_before_block_elem(&mut self.info)?;
            }
            YamlStyle::Flow => {
                if !self.info.is_first() {
                    self.ser.writer.write_ascii(", ")?;
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn end_seq_elem(&mut self) -> Result<(), Error> {
        self.info.state = CompoundState::Rest;
        match self.style {
            YamlStyle::Block => self.ser.write_after_block_elem()?,
            YamlStyle::Explicit => self.ser.writer.writeln()?,
            YamlStyle::Flow => {}
        }
        Ok(())
    }

    #[inline]
    fn begin_object(&mut self) -> Result<(), Error> {
        // Flush
        self.ser.is_scalar = false;
        self.push_state_in_seq()?;

        self.ser.flush_block_value()?;

        match self.style {
            YamlStyle::Block | YamlStyle::Explicit => {}
            YamlStyle::Flow => {
                self.ser.writer.write_ascii("{")?;
                self.ser.writer.write_n_spaces(1)?;
            }
        }
        Ok(())
    }

    #[inline]
    fn end_object(&mut self) -> Result<(), Error> {
        if self.style == YamlStyle::Flow {
            self.ser.writer.write_n_spaces(1)?;
            self.ser.writer.write_ascii("}")?;
        }
        self.ser.serializer_states.pop();

        Ok(())
    }

    #[inline]
    fn begin_obj_key(&mut self) -> Result<(), Error> {
        self.go_to_key();
        match self.style {
            YamlStyle::Block | YamlStyle::Explicit => {
                self.ser.ensure_indent()?;
            }
            YamlStyle::Flow if !self.info.is_first() => {
                self.ser.writer.write_ascii(", ")?;
            }
            YamlStyle::Flow => {
                self.info.state = CompoundState::Rest;
            }
        }
        Ok(())
    }

    #[inline]
    fn begin_obj_val(&mut self) {
        self.go_to_value();
        self.ser.key_val_sep = Some(self.ser.serializer_state().to_compound_style());
    }

    #[inline]
    fn go_to_value(&mut self) {
        *self.ser.serializer_state_mut() = match self.ser.serializer_state() {
            SerializerState::ExplicitKey | SerializerState::ExplicitValue => {
                SerializerState::ExplicitValue
            }
            SerializerState::Flow | SerializerState::FlowKey | SerializerState::FlowValue => {
                SerializerState::FlowValue
            }
            SerializerState::Block | SerializerState::BlockValue | SerializerState::BlockKey => {
                SerializerState::BlockValue
            }
        }
    }

    #[inline]
    fn go_to_key(&mut self) {
        *self.ser.serializer_state_mut() = match self.ser.serializer_state() {
            SerializerState::Block | SerializerState::BlockValue | SerializerState::BlockKey => {
                SerializerState::BlockKey
            }
            SerializerState::FlowValue | SerializerState::FlowKey | SerializerState::Flow => {
                SerializerState::FlowKey
            }
            SerializerState::ExplicitValue | SerializerState::ExplicitKey => {
                SerializerState::ExplicitKey
            }
        }
    }
}

impl<W> SerializeSeq for Compound<'_, W>
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
        self.begin_seq_elem()?;
        value.serialize(&mut *self.ser)?;
        self.end_seq_elem()
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_seq()?;
        Ok(())
    }
}

impl<W> ser::SerializeTuple for Compound<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_seq()
    }
}

impl<W> ser::SerializeTupleStruct for Compound<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_seq()
    }
}

impl<W> ser::SerializeTupleVariant for Compound<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_seq()?;
        self.end_object()
    }
}

impl<W> SerializeMap for Compound<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.begin_obj_key()?;
        key.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.begin_obj_val();
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_object()?;
        Ok(())
    }
}

impl<W> ser::SerializeStruct for Compound<'_, W>
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
        self.serialize_value(value)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_object()
    }
}

impl<W> SerializeStructVariant for Compound<'_, W>
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

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.end_object()
    }
}

#[cfg(test)]
mod test {
    use crate::node::SpanYam;
    use crate::prelude::{Marker, ScalarType, Span, YamLoader, YamlDocAccess};
    use crate::serde::ser::FlowStyle;
    use crate::serde::{NullFormat, YamlFormatter};
    use crate::to_formatted_string;
    use alloc::collections::BTreeMap;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use core::fmt::Error;
    use serde::Serialize;

    fn assert_eq_strings(result: Result<String, Error>, correct: &str) {
        assert_eq!(result, Ok(correct.to_string()));
    }
    const VEC_EXPECTED: &str = "- xyz\n- abc\n";

    #[test]
    fn test_vec_str() {
        let x = vec!["xyz".to_string(), "abc".to_string()];
        let fmt = YamlFormatter::default();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, VEC_EXPECTED);
    }

    #[test]
    fn test_null_fmt() {
        let x: Option<i32> = None;
        let fmt = YamlFormatter::default();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::Plain.to_null_string().as_ref());

        let fmt = YamlFormatter::pretty();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::JsonNull.to_null_string().as_ref());

        let mut fmt = YamlFormatter::default();
        fmt.set_null_format(NullFormat::OldYaml);
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::OldYaml.to_null_string().as_ref());

        let mut fmt = YamlFormatter::default();
        fmt.set_null_format(NullFormat::TaggedYaml);
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::TaggedYaml.to_null_string().as_ref());
    }

    const MULTI_LINE_STRING1_ACTUAL: &str = "One quick brown fox jumps over the lazy dog";
    const MULTI_LINE_STRING1_EXPECTED: &str = r#""One quick
brown fox
jumps over
the lazy
dog""#;
    #[test]
    fn test_multiline_string() {
        let formatter = {
            let mut x = YamlFormatter::pretty();
            x.pref_string_length = 10;
            x
        };
        let result = to_formatted_string(&MULTI_LINE_STRING1_ACTUAL, formatter);
        assert_eq!(result, Ok(MULTI_LINE_STRING1_EXPECTED.to_string()));
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Serialize)]
        struct Example;

        let formatter = YamlFormatter::pretty();
        let result = to_formatted_string(&Example, formatter);
        assert_eq!(result, Ok("{}".to_string()));
    }

    #[test]
    fn test_serialize_newtype_struct() {
        #[derive(Serialize)]
        struct Measurement(u8);

        let formatter = YamlFormatter::pretty();
        let result = to_formatted_string(&Measurement(0), formatter);
        assert_eq!(result, Ok("0".to_string()));
    }

    const SIMPLE_MAP_EXPECTED: &str = r#"a: 34
b: 1"#;

    #[test]
    fn test_serialize_simple_key() {
        let key = "a";
        let value = 34;
        let mut map = BTreeMap::new();
        map.insert(key, value);
        map.insert("b", 1);

        let result = to_formatted_string(&map, YamlFormatter::pretty());
        assert_eq!(result, Ok(SIMPLE_MAP_EXPECTED.to_string()));
    }

    const SIMPLE_STRUCT_EXPECTED: &str = r#"num: 3
string: test"#;

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize, Eq, Hash, PartialEq)]
        struct Example {
            num: i16,
            string: String,
        }

        let strct = Example {
            num: 3,
            string: "test".to_string(),
        };
        let result = to_formatted_string(&strct, YamlFormatter::pretty());
        assert_eq!(result, Ok(SIMPLE_STRUCT_EXPECTED.to_string()));
    }

    const LIST_EXPECTED: &str = r#"- 2
- 3
- 4
"#;

    #[test]
    fn test_list() {
        let example = vec![2, 3, 4];
        let result = to_formatted_string(&example, YamlFormatter::pretty());
        assert_eq!(result, Ok(LIST_EXPECTED.to_string()));
    }

    const NESTED_LIST_EXPECTED: &str = r#"- - 1
  - 2
- - 3
  - 4
"#;

    #[test]
    fn test_nested_list() {
        let example = vec![vec![1u64, 2], vec![3, 4]];

        let result = to_formatted_string(&example, YamlFormatter::pretty());
        assert_eq!(result, Ok(NESTED_LIST_EXPECTED.to_string()));
    }

    const COMPLEX_KEY_EXPECTED: &str = r#"? - 1
  - 2
: 34"#;

    #[test]
    fn test_serialize_complex_key() {
        let key = vec![1i64, 2];
        let value = 34u32;
        let map = BTreeMap::from([(key, value)]);

        let result = to_formatted_string(&map, YamlFormatter::pretty());
        assert_eq!(result, Ok(COMPLEX_KEY_EXPECTED.to_string()));
    }

    #[test]
    fn test_cant_be_plain() {
        let str = "@cant_be_plain";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.flow_string_style = FlowStyle::DoubleQuote;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"@cant_be_plain\"");

        let str = "@cant_be_plain";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.flow_string_style = FlowStyle::SingleQuote;

        let result = crate::to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "'@cant_be_plain'");
    }

    const FOLD_EXPECTED_1: &str = r#">-
block of text !!!"#;

    const FOLD_EXPECTED_2: &str = r#">
block of text !!!"#;

    const LIT_EXPECTED_1: &str = r#"|-
block of text !!!"#;

    const LIT_EXPECTED_2: &str = r#"|
block of text !!!"#;

    #[test]
    fn test_blocks_text() {
        let str = "block of text !!!";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Folded;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FOLD_EXPECTED_1);

        let str = "block of text !!!\n";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Folded;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FOLD_EXPECTED_2);

        let str = "block of text !!!";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Literal;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LIT_EXPECTED_1);

        let str = "block of text !!!\n";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Literal;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LIT_EXPECTED_2);
    }

    #[test]
    fn test_bool_char() {
        #[derive(Serialize)]
        enum Example {
            Bool((bool, bool)),
            Char(char),
        }

        let style = YamlFormatter::default();
        let result = to_formatted_string(&Example::Bool((true, false)), style);
        assert_eq!(result.unwrap(), "{ Bool: [ true, false ] }");

        let style = YamlFormatter::default();
        let result = to_formatted_string(&Example::Char('b'), style);
        assert_eq!(result.unwrap(), r#"{ Char: "b" }"#);
    }

    #[derive(Serialize)]
    struct Rgb(u8, u8, u8);

    #[test]
    fn test_tuple_struct() {
        let style = YamlFormatter::pretty();
        let result = to_formatted_string(&Rgb(0, 2, 4), style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[ 0, 2, 4 ]");
    }

    #[test]
    fn test_single_line_str() {
        let string = "AAA";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AAA");

        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::DoubleQuote;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"AAA\"");

        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::SingleQuote;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "'AAA'");
    }

    #[test]
    fn test_span_seq() {
        let input = r#"[
    a,
    b
]"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let first = &yaml[0];
            let second = &yaml[1];
            assert_eq!(first.as_str().unwrap_or(""), "a");
            assert_eq!(second.as_str().unwrap_or(""), "b");
            let span_1 = Span::new(Marker::new(6, 5, 2), Marker::new(7, 6, 2));
            let span_2 = Span::new(Marker::new(13, 5, 3), Marker::new(14, 6, 3));
            assert_eq!(first.span, span_1);
            assert_eq!(second.span, span_2);
        } else {
            panic!("Error parsing Yaml");
        }
    }

    #[test]
    fn test_span_map() {
        let input = r#"
a: 'aaa'
b: !!null
"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let first = &yaml["a"];
            let second = &yaml["b"];
            assert!(first.is_string(), "First value should be a string");
            assert_eq!(first.as_str().unwrap_or(""), "aaa");
            assert!(second.is_null(), "second element is null");
            let span_1 = Span::new(Marker::new(4, 4, 2), Marker::new(9, 9, 2));
            let span_2 = Span::new(Marker::new(20, 1, 4), Marker::new(20, 1, 4));
            assert_eq!(first.span, span_1);
            assert_eq!(second.span, span_2);
        } else {
            panic!("Error parsing Yaml");
        }
    }

    #[test]
    fn test_nested_map() {
        let input = r#"
a: !!vec [2, 3]
"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let vec = &yaml["a"];
            assert!(vec.is_non_empty_collection(), "Vec should be populated");
            let first = &vec[0];
            assert!(first.is_integer(), "First should be an integer");
            let second = &vec[1];
            assert!(second.is_integer(), "First should be an integer");
            assert_eq!(first.as_i64().unwrap_or(-1), 2);
            assert_eq!(second.as_i64().unwrap_or(-1), 3);
            let span_vec = Span::new(Marker::new(10, 10, 2), Marker::new(15, 15, 2));
            let span_first = Span::new(Marker::new(11, 11, 2), Marker::new(12, 12, 2));
            let span_second = Span::new(Marker::new(14, 14, 2), Marker::new(15, 15, 2));
            assert_eq!(vec.span, span_vec);
            assert_eq!(first.span, span_first);
            assert_eq!(second.span, span_second);
        } else {
            panic!("Error parsing Yaml");
        }
    }
}
