use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::str::{Utf8Error, from_utf8_unchecked};

pub type Mark = Range<usize>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ScalarType {
    /// Unquoted string type like:
    /// ```yaml
    ///   multiline
    ///   string
    /// ```
    Plain,
    /// Folded string type like:
    /// ```yaml
    ///   >
    ///     folded
    ///     string
    /// ```
    Folded,
    /// Folded string type like:
    /// ```yaml
    ///   |
    ///     folded
    ///     string
    /// ```
    Literal,
    /// Single quote string which permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// ' This is a quoted string
    ///    with ''quoted'' string within.'
    /// ```
    SingleQuote,
    /// Single quote string which permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// "This is a quoted string
    ///    with \"double quoted\" string within."
    /// ```
    DoubleQuote,
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::Plain => write!(f, ":"),
            ScalarType::Folded => write!(f, ">"),
            ScalarType::Literal => write!(f, "|"),
            ScalarType::SingleQuote => write!(f, "'"),
            ScalarType::DoubleQuote => write!(f, "\""),
        }
    }
}

#[derive(PartialEq)]
pub enum TokenType<'input> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    BlockSequenceStart,
    BlockMappingStart,
    BlockEnd,
    BlockEntry,
    FlowEntry,
    Key,
    Value,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    Alias(Cow<'input, str>),
    Anchor(Cow<'input, str>),
    VersionDirective {
        major: u8,
        minor: u8,
    },
    TagDirective {
        handle: Cow<'input, str>,
        suffix: Cow<'input, str>,
    },
    Scalar {
        scalar_type: ScalarType,
        value: Cow<'input, str>,
    },
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Marker {
    /// index in bytes of the input string.
    pub pos: usize,
    /// Column of mark. One indexed.
    pub col: u32,
    /// Column of mark. One indexed.
    pub line: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

#[derive(Clone, PartialEq)]
pub enum Event<'a> {
    DocStart,
    DocEnd,
    SeqStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
    },
    SeqEnd,
    MapStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
    },
    MapEnd,
    Directive {
        directive_type: DirectiveType,
        value: Cow<'a, [u8]>,
    },
    Scalar {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        scalar_type: ScalarType,
        value: Cow<'a, [u8]>,
    },
    Alias(Cow<'a, [u8]>),
    ErrorEvent,
}

impl Display for Event<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::DocStart => {
                write!(f, "+DOC")
            }
            Event::DocEnd => {
                write!(f, "-DOC")
            }
            Event::SeqStart { tag, anchor } => {
                write!(f, "+SEQ",)?;

                if let Some(cow) = anchor {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{string}")?;
                };
                if let Some(cow) = tag {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{string}>")?;
                };
                Ok(())
            }
            Event::SeqEnd => {
                write!(f, "-SEQ")
            }
            Event::MapStart { tag, anchor } => {
                write!(f, "+MAP")?;
                if let Some(cow) = anchor {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{string}")?;
                };
                if let Some(cow) = tag {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{string}>")?;
                };
                Ok(())
            }
            Event::MapEnd => {
                write!(f, "-MAP")
            }
            Event::Directive {
                directive_type,
                value,
            } => {
                // SAFETY:
                // SAFE as long as the slice is valid UTF8.
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                match directive_type {
                    DirectiveType::Yaml => write!(f, "%YAML {val_str}"),
                    _ => write!(f, "{val_str}"),
                }
            }
            Event::Scalar {
                scalar_type,
                value,
                tag,
                anchor,
            } => {
                // SAFETY:
                // SAFE as long as the slice is valid UTF8.
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                write!(f, "=VAL")?;

                if let Some(cow) = anchor {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string: &str = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " &{string}")?;
                };
                if let Some(cow) = tag {
                    // SAFETY:
                    // SAFE as long as the slice is valid UTF8.
                    let string = unsafe { from_utf8_unchecked(cow.as_ref()) };
                    write!(f, " <{string}>")?;
                };
                match *scalar_type {
                    ScalarType::Plain => write!(f, " :"),
                    ScalarType::Folded => write!(f, " >"),
                    ScalarType::Literal => write!(f, " |"),
                    ScalarType::SingleQuote => write!(f, " \'"),
                    ScalarType::DoubleQuote => write!(f, " \""),
                }?;
                write!(f, "{val_str}")?;

                Ok(())
            }
            Event::ErrorEvent => {
                write!(f, "ERR")
            }
            Event::Alias(value) => {
                // SAFETY:
                // SAFE as long as the slice is valid UTF8.
                let val_str = unsafe { from_utf8_unchecked(value.as_ref()) };
                write!(f, "=ALI *{val_str}")
            }
        }
    }
}

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type YamlResult<T> = Result<T, YamlError>;
pub type ScanResult = Result<(), YamlError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    Utf8(Utf8Error),
    Io(String),
    UnexpectedEof,
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
    ScannerErr {
        mark: Marker,
        info: String,
    },
}

impl YamlError {
    pub fn scanner_err(marker: Marker, info: &str) -> Self {
        YamlError::ScannerErr {
            mark: marker,
            info: info.to_string(),
        }
    }
}

impl From<Utf8Error> for YamlError {
    /// Creates a new `Error::NonDecodable` from the given error
    #[inline]
    fn from(error: Utf8Error) -> YamlError {
        YamlError::NonDecodable(Some(error))
    }
}
