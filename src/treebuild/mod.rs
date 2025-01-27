use std::borrow::Cow;
use std::fmt::Display;

use crate::tokenizer::ErrorType;
use crate::treebuild::YamlToken::Scalar;

pub enum YamlToken<'a, TAG = ()> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a, [u8]>, TAG),

    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Vec<YamlToken<'a, TAG>>, TAG),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Vec<Entry<'a, TAG>>, TAG),
}

impl<'a, TAG> Display for YamlToken<'a, TAG> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scalar(val, _) => write!(f, "SCAL: {:?}", val),
            Self::Sequence(val, _) => {
                write!(f, "SEQ:")?;
                for el in val {
                    write!(f, "{}", el)?;
                }
                Ok(())
            }
            Self::Mapping(val, _) => {
                write!(f, "MAP:")?;
                for entry in val {
                    write!(f, "{} = {}", entry.key, entry.value)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a, TAG: Default> Default for YamlToken<'a, TAG> {
    fn default() -> Self {
        Scalar(Cow::default(), TAG::default())
    }
}

pub struct Entry<'a, TAG> {
    key: YamlToken<'a, TAG>,
    value: YamlToken<'a, TAG>,
}

impl<'a, TAG: Default> Default for Entry<'a, TAG> {
    fn default() -> Self {
        Entry {
            key: YamlToken::default(),
            value: YamlToken::default(),
        }
    }
}

pub struct YamlTokenError<'a, T> {
    _partial: YamlToken<'a, T>,
    _error: Vec<ErrorType>,
}
