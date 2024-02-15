use alloc::string::String;
use core::str::Utf8Error;
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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from(error: Utf8Error) -> YamlError {
        YamlError::NonDecodable(Some(error))
    }
}
