use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ErrorType {}

#[derive(Debug, PartialEq)]
pub struct Error {
    /// Type of error
    error: ErrorType,
}

impl Error {
    pub fn generic(t: ErrorType) -> Self {
        Self { error: t }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

impl std::error::Error for Error {}
