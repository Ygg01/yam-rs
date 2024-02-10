
#[derive(Debug, PartialEq)]
pub enum ErrorType {

}

#[derive(Debug, PartialEq)]
pub struct Error {
    /// Type of error
    error: ErrorType,
}