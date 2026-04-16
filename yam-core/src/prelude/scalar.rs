use crate::prelude::{ScalarType, Tag};
use alloc::borrow::Cow;
use core::marker::PhantomData;

#[derive(Debug)]
pub enum YamlScalar<'a, F = f64, STR = Cow<'a, str>> {
    Null(PhantomData<&'a ()>),
    String(STR),
    Bool(bool),
    FloatingPoint(F),
    Integer(i64),
}

impl<'a, F, S> PartialEq for YamlScalar<'a, F, S>
where
    F: PartialEq,
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (YamlScalar::Null(_), YamlScalar::Null(_)) => true,
            (YamlScalar::String(s1), YamlScalar::String(s2)) => s1 == s2,
            (YamlScalar::Bool(s1), YamlScalar::Bool(s2)) => s1 == s2,
            (YamlScalar::Integer(s1), YamlScalar::Integer(s2)) => s1 == s2,
            (YamlScalar::FloatingPoint(s1), YamlScalar::FloatingPoint(s2)) => s1 == s2,
            (_, _) => false,
        }
    }
}

impl<'a, F> YamlScalar<'a, F>
where
    F: From<f64>,
{
    /// Parse a scalar node representation into a [`Scalar`].
    ///
    /// If `tag` is not [`None`]:
    ///   - If the handle is `tag:yaml.org,2022:`, attempt to parse as the given suffix. If parsing
    ///     fails or the suffix is unknown, return [`None`].
    ///   - If the handle is unknown, use the fallback parsing schema.
    ///
    /// # Return
    /// Returns the parsed [`Scalar`].
    ///
    pub fn parse_from_cow_and_metadata(
        v: Cow<'a, str>,
        style: ScalarType,
        tag: Option<&Cow<'a, Tag>>,
    ) -> Option<Self> {
        if style != ScalarType::Plain {
            // Any quoted scalar is a string.
            Some(Self::String(v.into()))
        } else if let Some(tag) = tag.map(Cow::as_ref) {
            if tag.is_yaml_core_schema() {
                match tag.suffix.as_ref() {
                    "bool" => v.parse::<bool>().ok().map(|x| Self::Bool(x)),
                    "int" => v.parse::<i64>().ok().map(|x| Self::Integer(x)),
                    "float" => parse_core_schema_fp(&v).map(|x| Self::FloatingPoint(x.into())),
                    "null" => match v.as_ref() {
                        "~" | "null" => Some(Self::Null(PhantomData::default())),
                        _ => None,
                    },
                    "str" => Some(Self::String(v.into())),
                    // If we have a tag we do not recognize, return `None`.
                    _ => None,
                }
            } else {
                // If we have a tag we do not recognize, parse it regularly.
                // This will sound more intuitive when instance reading tagged scalars like
                // `!degree 50`.
                Some(Self::parse_from_cow(v))
            }
        } else {
            // No tag means we have to guess.
            Some(Self::parse_from_cow(v))
        }
    }

    /// Parse a scalar node representation into a [`Scalar`].
    ///
    /// This function cannot fail. It will fallback to [`Scalar::String`] if everything else fails.
    ///
    /// # Return
    /// Returns the parsed [`Scalar`].
    #[must_use]
    pub fn parse_from_cow(v: Cow<'a, str>) -> Self {
        let s = &*v;
        let bytes = s.as_bytes();

        if bytes.len() >= 2 {
            match (bytes[0], bytes[1]) {
                (b'0', b'x') => {
                    if let Ok(i) = i64::from_str_radix(&s[2..], 16) {
                        return Self::Integer(i);
                    }
                }
                (b'0', b'o') => {
                    if let Ok(i) = i64::from_str_radix(&s[2..], 8) {
                        return Self::Integer(i);
                    }
                }
                (b'+', _) => {
                    if let Ok(i) = s[1..].parse::<i64>() {
                        return Self::Integer(i);
                    }
                }
                _ => {}
            }
        }

        match bytes.len() {
            1 if bytes[0] == b'~' => return Self::Null(PhantomData::default()),
            4 => {
                let f = bytes[0] & 0xDF;
                if f == b'N' && matches!(s, "null" | "Null" | "NULL") {
                    return Self::Null(PhantomData::default());
                } else if f == b'T' && matches!(s, "true" | "True" | "TRUE") {
                    return Self::Bool(true);
                }
            }
            5 if matches!(s, "false" | "False" | "FALSE") => {
                return Self::Bool(false);
            }
            _ => {}
        }

        if let Ok(integer) = s.parse::<i64>() {
            return Self::Integer(integer);
        }

        if let Some(float) = parse_core_schema_fp(s) {
            return Self::FloatingPoint(float.into());
        }

        Self::String(v.into())
    }
}

/// Parse the given string as a floating point according to the core schema.
///
/// See [10.2.1.4](https://yaml.org/spec/1.2.2/#10214-floating-point) for the floating point
/// definition.
///
/// # Return
/// Returns `Some` if parsing succeeding, `None` otherwise. This function is used in the process of
/// parsing scalars, where failing to parse a scalar as a floating point is not an error. As such,
/// this function purposefully does not return a `Result`.
pub fn parse_core_schema_fp(v: &str) -> Option<f64> {
    match v {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => Some(f64::NAN),
        // Test that `v` contains a digit so as not to pass in strings like `inf`,
        // which rust will parse as a float.
        _ if v.as_bytes().iter().any(u8::is_ascii_digit) => v.parse::<f64>().ok(),
        _ => None,
    }
}

impl<F> Clone for YamlScalar<'_, F>
where
    F: Copy,
{
    fn clone(&self) -> Self {
        match self {
            YamlScalar::Null(a) => YamlScalar::Null(*a),
            YamlScalar::String(s) => YamlScalar::String(s.clone()),
            YamlScalar::FloatingPoint(f) => YamlScalar::FloatingPoint(*f),
            YamlScalar::Bool(b) => YamlScalar::Bool(*b),
            YamlScalar::Integer(i) => YamlScalar::Integer(*i),
        }
    }
}
