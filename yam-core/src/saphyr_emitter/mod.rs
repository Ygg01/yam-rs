use crate::prelude::YamlData;
use crate::prelude::{
    IsEmpty, MappingLike, SequenceLike, YamlDocAccess, YamlScalar, is_valid_literal_block_scalar,
};
use crate::prelude::{Yaml, YamlEntry};
use alloc::vec::Vec;
use core::borrow::BorrowMut;
use core::fmt;
use core::marker::PhantomData;

#[allow(clippy::module_name_repetitions)]
pub struct YamlEmitter<'a, FP> {
    writer: &'a mut dyn fmt::Write,
    best_indent: usize,
    compact: bool,
    level: isize,
    multiline_strings: bool,
    _marker: PhantomData<FP>,
}

/// A convenience alias for emitter functions that may fail without returning a value.
pub type EmitResult = Result<(), fmt::Error>;

// from serialize::json
fn escape_str(wr: &mut dyn fmt::Write, v: &str) -> EmitResult {
    wr.write_str("\"")?;

    let bytes = v.as_bytes();
    let mut start = 0;

    while start < bytes.len() {
        let Some(i) = bytes[start..]
            .iter()
            .position(|&b| matches!(b, b'"' | b'\\' | b'\x00'..=b'\x1f' | b'\x7f'))
        else {
            wr.write_str(&v[start..])?;
            break;
        };

        let i = start + i;

        if start < i {
            wr.write_str(&v[start..i])?;
        }

        let escaped = match bytes[i] {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => unreachable!(),
        };

        wr.write_str(escaped)?;
        start = i + 1;
    }

    wr.write_str("\"")?;
    Ok(())
}

#[allow(dead_code)]
impl<'a, FP> YamlEmitter<'a, FP>
where
    // STR: Clone + for<'x> From<&'x str> + AsRef<str> + ToMutStr + Into<String>,
    FP: Copy + BorrowMut<f64> + Into<f64>,
{
    /// Create a new emitter serializing into `writer`.
    pub fn new(writer: &'a mut dyn fmt::Write) -> Self {
        YamlEmitter {
            writer,
            best_indent: 2,
            compact: true,
            level: -1,
            multiline_strings: false,
            _marker: PhantomData,
        }
    }

    /// Set 'compact inline notation' on or off, as described for block
    /// [sequences](http://www.yaml.org/spec/1.2/spec.html#id2797382)
    /// and
    /// [mappings](http://www.yaml.org/spec/1.2/spec.html#id2798057).
    ///
    /// In this form, blocks cannot have any properties (such as anchors
    /// or tags), which should be OK, because this emitter doesn't
    /// (currently) emit those anyways.
    ///
    /// TODO(ethiraric, 2024/04/02): We can support those now.
    pub fn compact(&mut self, compact: bool) {
        self.compact = compact;
    }

    /// Determine if this emitter is using 'compact inline notation'.
    #[must_use]
    pub fn is_compact(&self) -> bool {
        self.compact
    }

    ///
    /// Sets the `multiline_strings` property for the current instance.
    ///
    /// This function allows enabling or disabling the handling of multiline strings.
    ///
    /// # Parameters
    /// - `multiline_strings` (bool): A boolean value indicating whether multiline strings
    ///   should be enabled (`true`) or disabled (`false`).
    ///
    /// # Note
    /// This method mutates the current instance by changing the `multiline_strings` field.
    ///
    pub fn multiline_strings(&mut self, multiline_strings: bool) {
        self.multiline_strings = multiline_strings;
    }

    /// Determine if this emitter will emit multiline strings when appropriate.
    #[must_use]
    pub fn is_multiline_strings(&self) -> bool {
        self.multiline_strings
    }

    /// Dump Yaml to an output stream.
    /// # Errors
    /// Returns `EmitError` when an error occurs.
    pub fn dump(&mut self, doc: &Yaml<'a, FP>) -> EmitResult {
        // write DocumentStart
        writeln!(self.writer, "---")?;
        self.level = -1;
        self.emit_node(doc)
    }

    fn write_indent(&mut self) -> EmitResult {
        if self.level <= 0 {
            return Ok(());
        }
        for _ in 0..self.level {
            for _ in 0..self.best_indent {
                write!(self.writer, " ")?;
            }
        }
        Ok(())
    }

    fn emit_node(&mut self, node: &Yaml<'a, FP>) -> EmitResult {
        match &node.0 {
            YamlData::Sequence(v) => self.emit_sequence(v),
            YamlData::Mapping(h) => self.emit_mapping(h),
            YamlData::Scalar(YamlScalar::String(v)) => {
                if self.should_emit_string_as_block(v.as_ref()) {
                    self.emit_literal_block(v.as_ref())?;
                } else if need_quotes(v.as_ref()) {
                    escape_str(self.writer, v.as_ref())?;
                } else {
                    write!(self.writer, "{0}", v.as_ref())?;
                }
                Ok(())
            }
            YamlData::Scalar(YamlScalar::Bool(v)) => {
                if *v {
                    self.writer.write_str("true")?;
                } else {
                    self.writer.write_str("false")?;
                }
                Ok(())
            }
            YamlData::Scalar(YamlScalar::Integer(v)) => Ok(write!(self.writer, "{v}")?),
            YamlData::Scalar(YamlScalar::FloatingPoint(v)) => {
                Ok(write!(self.writer, "{0}", <FP as Into<f64>>::into(*v))?)
            }
            YamlData::BadValue | YamlData::Scalar(YamlScalar::Null(_)) => {
                Ok(write!(self.writer, "~")?)
            }
            YamlData::Tagged(tag, node) => {
                write!(self.writer, "{} ", tag.as_ref())?;
                // We need to insert a newline after the tag in the following cases:
                //   - We have a non-empty sequence or mapping. `emit_sequence` and `emit_mapping`
                //     do not add that extra newline at the beginning.
                //       foo: !tag {} // OK
                //       ---
                //       foo: !tag [] // OK
                //       ---
                //       foo: !tag bar: baz // KO
                //       ---
                //       foo: !tag // OK
                //         bar: baz
                //       ---
                //       foo: !tag - a // OK
                //       ---
                //       foo: !tag - a // KO
                //         - b
                //       ---
                //       foo: !tag // OK
                //         - a
                //         - b
                if node.is_non_empty_collection() {
                    self.level += 1;
                    writeln!(self.writer)?;
                    self.write_indent()?;
                    self.level -= 1;
                }
                self.emit_node(node.as_ref())
            }
            // XXX(chenyh) Alias
            YamlData::Alias(_) => Ok(()),
        }
    }

    fn emit_literal_block(&mut self, v: &str) -> EmitResult {
        let ends_with_newline = v.ends_with('\n');
        if ends_with_newline {
            self.writer.write_str("|")?;
        } else {
            self.writer.write_str("|-")?;
        }

        self.level += 1;
        // lines() will omit the last line if it is empty.
        for line in v.lines() {
            writeln!(self.writer)?;
            self.write_indent()?;
            // It's literal text, so don't escape special chars.
            self.writer.write_str(line)?;
        }
        self.level -= 1;
        Ok(())
    }

    fn emit_sequence(&mut self, v: &Vec<Yaml<'a, FP>>) -> EmitResult {
        if v.is_collection_empty() {
            write!(self.writer, "[]")?;
        } else {
            self.level += 1;
            for (cnt, x) in v.vec().iter().enumerate() {
                if cnt > 0 {
                    writeln!(self.writer)?;
                    self.write_indent()?;
                }
                write!(self.writer, "-")?;
                self.emit_val(true, x)?;
            }
            self.level -= 1;
        }
        Ok(())
    }

    fn emit_mapping(&mut self, h: &Vec<YamlEntry<'a, Yaml<'a, FP>>>) -> EmitResult {
        if h.is_collection_empty() {
            self.writer.write_str("{}")?;
        } else {
            self.level += 1;
            for (cnt, entry) in h.entries().iter().enumerate() {
                let complex_key =
                    matches!(entry.key.0, YamlData::Mapping(_) | YamlData::Sequence(_));
                if cnt > 0 {
                    writeln!(self.writer)?;
                    self.write_indent()?;
                }
                if complex_key {
                    write!(self.writer, "?")?;
                    self.emit_val(true, &entry.key)?;
                    writeln!(self.writer)?;
                    self.write_indent()?;
                    write!(self.writer, ":")?;
                    self.emit_val(true, &entry.value)?;
                } else {
                    self.emit_node(&entry.key)?;
                    write!(self.writer, ":")?;
                    self.emit_val(false, &entry.value)?;
                }
            }
            self.level -= 1;
        }
        Ok(())
    }

    /// Emit a yaml as a hash or array value: i.e., which should appear
    /// following a ":" or "-", either after a space or on a new line.
    /// If `inline` is true, then the preceding characters are distinct
    /// and short enough to respect the compact flag.
    fn emit_val(&mut self, inline: bool, val: &Yaml<'a, FP>) -> EmitResult {
        macro_rules! write_collection {
            ($v:expr ) => {
                if (inline && self.compact) || $v.is_collection_empty() {
                    write!(self.writer, " ")?;
                } else {
                    writeln!(self.writer)?;
                    self.level += 1;
                    self.write_indent()?;
                    self.level -= 1;
                }
            };
        }

        match val.0 {
            YamlData::Sequence(ref v) => {
                write_collection!(v);
                self.emit_sequence(v)
            }
            YamlData::Mapping(ref v) => {
                write_collection!(v);
                self.emit_mapping(v)
            }
            _ => {
                write!(self.writer, " ")?;
                self.emit_node(val)
            }
        }
    }

    /// Check whether the string should be emitted as a literal block.
    ///
    /// This takes into account the [`multiline_strings`] option and whether the string contains
    /// newlines.
    ///
    /// [`multiline_strings`]: Self::multiline_strings.
    #[must_use]
    fn should_emit_string_as_block(&self, s: &str) -> bool {
        self.multiline_strings && s.contains('\n') && is_valid_literal_block_scalar(s)
    }
}

/// Check if the string requires quoting.
/// Strings starting with any of the following characters must be quoted.
/// `:`, `&`, `*`, `?`, `|`, `-`, `<`, `>`, `=`, `!`, `%`, `@`
/// Strings containing any of the following characters must be quoted.
/// `{`, `}`, `[`, `]`, `,`, `#`, `` ` ``, `"`, `'`,
///
/// If the string contains any of the following control characters, it must be escaped with double quotes:
/// `\x00`, `\x01`, `\x02`, `\x03`, `\x04`, `\x05`, `\x06`, `\a`, `\b`, `\t`, `\n`, `\v`, `\f`, `\r`,
///  `\x0E`, `\x0F`, `\x10`, `\x11`, `\x12`, `\x13`, `\x14`, `\x15`, `\x16`, `\x17`, `\x18`,
///  `\x19`, `\x1A`, `\x0E`, `\x1C`, `\x1D`, `\x1E`, `\x1F`
///
/// Finally, there are other cases when the strings must be quoted, no matter if you're using single or double quotes:
/// * When the string is true or false (otherwise, it would be treated as a boolean value);
/// * When the string is `null` or `~` (otherwise, it would be considered as a null value);
/// * When the string looks like a number, such as integers (e.g., `2`, `14`, etc.), floats (e.g., `2.6`, `14.9`)
///   and exponential numbers (e.g., `12e7`, etc.) (otherwise, it would be treated as a numeric value);
/// * When the string looks like a date (e.g. `2014-12-31`) (otherwise it would be automatically converted into a Unix timestamp).
#[allow(clippy::doc_markdown)]
fn need_quotes(string: &str) -> bool {
    fn need_quotes_spaces(string: &str) -> bool {
        string.starts_with(' ') || string.ends_with(' ')
    }

    string.is_empty()
        || need_quotes_spaces(string)
        || string.starts_with(|character: char| {
            matches!(
                character,
                '&' | '*' | '?' | '|' | '-' | '<' | '>' | '=' | '!' | '%' | '@'
            )
        })
        || string.contains(|character: char| {
            matches!(character, ':'
            | '{'
            | '}'
            | '['
            | ']'
            | ','
            | '#'
            | '`'
            | '\"'
            | '\''
            | '\\'
            | '\0'..='\x06'
            | '\t'
            | '\n'
            | '\r'
            | '\x0e'..='\x1a'
            | '\x1c'..='\x1f')
        })
        || [
            // http://yaml.org/type/bool.html
            // Note: 'y', 'Y', 'n', 'N', is not quoted deliberately, as in libyaml. PyYAML also parses
            // them as string, not booleans, although it is violating the YAML 1.1 specification.
            // See https://github.com/dtolnay/serde-yaml/pull/83#discussion_r152628088.
            "yes", "Yes", "YES", "no", "No", "NO", "True", "TRUE", "true", "False", "FALSE",
            "false", "on", "On", "ON", "off", "Off", "OFF",
            // http://yaml.org/type/null.html
            "null", "Null", "NULL", "~",
        ]
        .contains(&string)
        || string.starts_with('.')
        || string.starts_with("0x")
        || string.parse::<i64>().is_ok()
        || string.parse::<f64>().is_ok()
}
