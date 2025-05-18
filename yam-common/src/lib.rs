use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::{Utf8Error, from_utf8_unchecked};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ScalarType {
    Plain,
    Folded,
    Literal,
    SingleQuote,
    DoubleQuote,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

#[derive(Clone, PartialEq)]
pub enum Event<'a> {
    DocStart {
        explicit: bool,
    },
    DocEnd {
        explicit: bool,
    },
    SeqStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        flow: bool,
    },
    SeqEnd,
    MapStart {
        tag: Option<Cow<'a, [u8]>>,
        anchor: Option<Cow<'a, [u8]>>,
        flow: bool,
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
            Event::DocStart { explicit } => {
                let exp_str = if *explicit { " ---" } else { "" };
                write!(f, "+DOC{exp_str}")
            }
            Event::DocEnd { explicit } => {
                let exp_str = if *explicit { " ..." } else { "" };
                write!(f, "-DOC{exp_str}")
            }
            Event::SeqStart { flow, tag, anchor } => {
                write!(f, "+SEQ",)?;
                if *flow {
                    write!(f, " []")?;
                }
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
            Event::MapStart { flow, tag, anchor } => {
                write!(f, "+MAP")?;
                if *flow {
                    write!(f, " {{}}")?;
                }
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    Utf8(Utf8Error),
    Io(String),
    UnexpectedEof,
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
}

impl From<Utf8Error> for YamlError {
    /// Creates a new `Error::NonDecodable` from the given error
    #[inline]
    fn from(error: Utf8Error) -> YamlError {
        YamlError::NonDecodable(Some(error))
    }
}
