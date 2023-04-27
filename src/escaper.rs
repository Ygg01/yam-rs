use std::{borrow::Cow, ops::Deref};

pub fn escape_plain(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(input, |ch| matches!(ch, b'\n' | b'\t' | b'\\'))
}

pub fn escape_quotes(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(input, |ch| matches!(ch, b'\n' | b'\t' | b'\\'))
}

pub(crate) fn _escape<F: Fn(u8) -> bool>(input: Cow<'_, [u8]>, escape_fn: F) -> Cow<'_, [u8]> {
    let raw = input.deref();
    let mut iter = raw.iter();
    let mut pos = 0;
    let mut escaped: Option<Vec<u8>> = None;
    while let Some(i) = iter.position(|&b| escape_fn(b)) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("Initialized");
        let new_pos = pos + i;
        escaped.extend(&raw[pos..new_pos]);
        match raw[new_pos] {
            b'\\' => escaped.extend("\\\\".as_bytes()),
            b'\t' => escaped.extend("\\t".as_bytes()),
            b'\r' => escaped.extend("\\r".as_bytes()),
            b'\n' => escaped.extend("\\n".as_bytes()),
            b'\'' => escaped.extend("\\'".as_bytes()),
            b'"' => escaped.extend("\\\"".as_bytes()),
            _ => unreachable!("Only '\' are escaped"),
        }
        pos = new_pos + 1;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = raw.get(pos..) {
            escaped.extend(raw);
        }
        Cow::Owned(escaped)
    } else {
        input
    }
}
