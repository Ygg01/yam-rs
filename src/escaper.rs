use std::{borrow::Cow, ops::Deref};

pub fn escape_plain(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n',
        |input, pos, escaped| {
            return match input {
                [b'\\', b't', ..] | [b'\\', b'r', ..] | [b'\\', b'n', ..] => pos,
                [b'\r', ..] => {
                    escaped.extend("\\r".as_bytes());
                    pos + 1
                }
                [b'\t', ..] => {
                    escaped.extend("\\t".as_bytes());
                    pos + 1
                }
                [b'\n', ..] => {
                    escaped.extend("\\n".as_bytes());
                    pos + 1
                }
                [b'\\', ..] => {
                    escaped.extend("\\\\".as_bytes());
                    pos + 1
                }
                [b'\'', ..] => {
                    escaped.extend("\\'".as_bytes());
                    pos + 1
                }
                _ => unreachable!("Only '\' are escaped"),
            };
        },
    )
}

pub fn escape_double_quotes(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n' || chr == b'\r',
        |input, pos, escaped| {
            return match input {
                [b'\\', b't', ..] | [b'\\', b'r', ..] | [b'\\', b'n', ..] => pos,
                [b'\r', ..] => {
                    escaped.extend("\\r".as_bytes());
                    pos + 1
                }
                [b'\t', ..] => {
                    escaped.extend("\\t".as_bytes());
                    pos + 1
                }
                [b'\n', ..] => {
                    escaped.extend("\\n".as_bytes());
                    pos + 1
                }
                [b'\\', ..] => {
                    escaped.extend("\\\\".as_bytes());
                    pos + 1
                }
                _ => unreachable!("Only '\' are escaped"),
            };
        },
    )
}

fn _escape<F: Fn(&u8) -> bool, M: FnMut(&[u8], usize, &mut Vec<u8>) -> usize>(
    input: Cow<[u8]>,
    find_fn: F,
    mut match_fn: M,
) -> Cow<[u8]> {
    let raw = input.deref();
    let mut iter = raw.iter();
    let mut pos = 0;
    let mut escaped: Option<Vec<u8>> = None;
    while let Some(i) = iter.position(&find_fn) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let mut escaped = escaped.as_mut().expect("Initialized");
        let new_pos = pos + i;
        escaped.extend(&raw[pos..new_pos]);
        pos = match_fn(&raw[new_pos..], new_pos, &mut escaped);
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
