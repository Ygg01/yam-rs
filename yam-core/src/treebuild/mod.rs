use alloc::borrow::Cow;
use alloc::fmt::Display;
use alloc::vec::Vec;

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

impl<TAG> Display for YamlToken<'_, TAG> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Scalar(val, _) => write!(f, "SCAL: {val:?}"),
            Self::Sequence(val, _) => {
                write!(f, "SEQ:")?;
                for el in val {
                    write!(f, "{el}")?;
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

impl<TAG: Default> Default for YamlToken<'_, TAG> {
    fn default() -> Self {
        Scalar(Cow::default(), TAG::default())
    }
}

pub struct Entry<'a, TAG> {
    key: YamlToken<'a, TAG>,
    value: YamlToken<'a, TAG>,
}

impl<TAG: Default> Default for Entry<'_, TAG> {
    fn default() -> Self {
        Entry {
            key: YamlToken::default(),
            value: YamlToken::default(),
        }
    }
}
