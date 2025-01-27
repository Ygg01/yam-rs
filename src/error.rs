use std::str::Utf8Error;

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type YamlResult<T> = std::result::Result<T, YamlError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    Utf8(Utf8Error),
    Io(String),
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
}

impl From<::std::io::Error> for YamlError {
    /// Creates a new `Error::Io` from the given error
    #[inline]
    fn from(error: ::std::io::Error) -> YamlError {
        YamlError::Io(error.to_string())
    }
}

impl From<Utf8Error> for YamlError {
    /// Creates a new `Error::NonDecodable` from the given error
    #[inline]
    fn from(error: Utf8Error) -> YamlError {
        YamlError::NonDecodable(Some(error))
    }
}