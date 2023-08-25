use std::borrow::Cow;

struct Escape<'a, F, M> {
    find_fn: F,
    match_fn: M,
    iter: &'a [u8],
    pos: usize,
}

impl<'a, F, S> Escape<'a, F, S> {
    pub(crate) fn new(iter: &'a [u8], find_fn: F, skip_fn: S) -> Escape<'a, F, S> {
        Escape {
            find_fn,
            match_fn: skip_fn,
            iter,
            pos: 0,
        }
    }
}

pub enum EscapeControl {
    Skip(u8),
    Append([u8; 8]),
    Break,
}

pub struct NewPos {
    pos: usize,
    step: u8,
    len: u8,
    bytes: [u8; 6],
}

impl<'a, F, M> Iterator for Escape<'a, F, M>
where
    F: Fn(&u8) -> bool,
    M: Fn(&[u8]) -> EscapeControl,
{
    type Item = NewPos;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(find_pos) = self.iter[self.pos..].iter().position(&self.find_fn) {
            match (self.match_fn)(&self.iter[self.pos + find_pos..]) {
                EscapeControl::Append(bytes) => {
                    let pos = self.pos + find_pos;
                    let step = bytes[0];
                    let len = bytes[1];
                    self.pos = pos + step as usize;
                    return Some(NewPos {
                        pos,
                        step,
                        len,
                        bytes: [bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]],
                    });
                }
                EscapeControl::Skip(x) => {
                    self.pos += x as usize + find_pos;
                    continue;
                }
                EscapeControl::Break => {
                    return None;
                }
            }
        }
        None
    }
}

#[must_use]
pub fn escape_plain(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n',
        |input| match input {
            [b'\\', b't' | b'r', ..] => EscapeControl::Skip(2),
            [b'\r', ..] => EscapeControl::Append([1, 2, b'\\', b'r', 0, 0, 0, 0]),
            [b'\t', ..] => EscapeControl::Append([1, 2, b'\\', b't', 0, 0, 0, 0]),
            [b'\n', ..] => EscapeControl::Append([1, 2, b'\\', b'n', 0, 0, 0, 0]),
            [b'\\', ..] => EscapeControl::Append([1, 2, b'\\', b'\\', 0, 0, 0, 0]),
            _ => EscapeControl::Break,
        },
    )
}

#[must_use]
pub fn escape_double_quotes(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n' || chr == b'\r',
        |input: &[u8]| match input {
            [b'\\', b't' | b'r' | b'n', ..] => EscapeControl::Skip(2),
            [b'\\', b'x', ..] => decode_hex(input, 2),
            [b'\\', b'u', ..] => decode_hex(input, 4),
            [b'\\', b'U', ..] => decode_hex(input, 8),
            [b'\\', b'/', ..] => EscapeControl::Append([2, 1, b'/', 0, 0, 0, 0, 0]),
            [b'\r', ..] => EscapeControl::Append([1, 2, b'\\', b'r', 0, 0, 0, 0]),
            [b'\t', ..] => EscapeControl::Append([1, 2, b'\\', b't', 0, 0, 0, 0]),
            [b'\n', ..] => EscapeControl::Append([1, 2, b'\\', b'n', 0, 0, 0, 0]),
            [b'\'', ..] => EscapeControl::Append([1, 2, b'\\', b'\'', 0, 0, 0, 0]),
            _ => EscapeControl::Break,
        },
    )
}

#[must_use]
pub fn escape_single_quotes(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n' || chr == b'\r',
        |input| match input {
            [b'\r', ..] => EscapeControl::Append([1, 2, b'\\', b'r', 0, 0, 0, 0]),
            [b'\t', ..] => EscapeControl::Append([1, 2, b'\\', b't', 0, 0, 0, 0]),
            [b'\n', ..] => EscapeControl::Append([1, 2, b'\\', b'n', 0, 0, 0, 0]),
            [b'\\', ..] => EscapeControl::Append([1, 2, b'\\', b'\\', 0, 0, 0, 0]),
            [b'\'', ..] => EscapeControl::Append([1, 2, b'\\', b'\'', 0, 0, 0, 0]),
            _ => EscapeControl::Break,
        },
    )
}

fn _escape<F, M>(input: Cow<[u8]>, find_fn: F, match_fn: M) -> Cow<[u8]>
where
    F: Fn(&u8) -> bool,
    M: Fn(&[u8]) -> EscapeControl,
{
    let raw = &*input;
    let escape_iter = Escape::new(raw, find_fn, match_fn);
    let mut old_pos = 0;
    let mut escaped: Option<Vec<u8>> = None;

    for NewPos {
        pos,
        step,
        len,
        bytes,
    } in escape_iter
    {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("Expected it to be initialized!");
        escaped.extend(&raw[old_pos..pos]);
        escaped.extend(&bytes[0..len as usize]);
        old_pos = pos + step as usize;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = raw.get(old_pos..) {
            escaped.extend(raw);
        }
        Cow::Owned(escaped)
    } else {
        input
    }
}

fn decode_hex(input: &[u8], size: u8) -> EscapeControl {
    // Manually encoded U+FFFD (Replacement characters)
    let replacement_char = [size + 2, 3, 239, 191, 189, 0, 0, 0];
    let digit_slice = input.iter().skip(2).take(size as usize);
    if !digit_slice.clone().all(u8::is_ascii_hexdigit) {
        return EscapeControl::Append(replacement_char);
    }

    let code_point = digit_slice
        .map(|x| match *x {
            n @ b'0'..=b'9' => n - b'0',
            a @ b'a'..=b'f' => a - b'a' + 10,
            a @ b'A'..=b'F' => a - b'A' + 10,
            _ => 0u8,
        })
        .fold(0u32, |acc, digit| (acc << 4) + digit as u32);
    match code_point {
        // YAML has special escape rules for certain values
        // See more in https://yaml.org/spec/1.2.2/#57-escaped-characters
        0 => return EscapeControl::Append([size + 2, 2, b'\\', b'0', 0, 0, 0, 0]),
        0x07 => return EscapeControl::Append([size + 2, 2, b'\\', b'a', 0, 0, 0, 0]),
        0x08 => return EscapeControl::Append([size + 2, 2, b'\\', b'b', 0, 0, 0, 0]),
        0x09 => return EscapeControl::Append([size + 2, 2, b'\\', b't', 0, 0, 0, 0]),
        0x0A => return EscapeControl::Append([size + 2, 2, b'\\', b'n', 0, 0, 0, 0]),
        0x0B => return EscapeControl::Append([size + 2, 2, b'\\', b'v', 0, 0, 0, 0]),
        0x0C => return EscapeControl::Append([size + 2, 2, b'\\', b'f', 0, 0, 0, 0]),
        0x0D => return EscapeControl::Append([size + 2, 2, b'\\', b'r', 0, 0, 0, 0]),
        0x1B => return EscapeControl::Append([size + 2, 2, b'\\', b'e', 0, 0, 0, 0]),
        0x20 => return EscapeControl::Append([size + 2, 2, b'\\', b' ', 0, 0, 0, 0]),
        0x22 => return EscapeControl::Append([size + 2, 2, b'\\', b'"', 0, 0, 0, 0]),
        0x2F => return EscapeControl::Append([size + 2, 2, b'\\', b'/', 0, 0, 0, 0]),
        0x5C => return EscapeControl::Append([size + 2, 2, b'\\', b'\\', 0, 0, 0, 0]),
        0x85 => return EscapeControl::Append([size + 2, 2, b'\\', b'N', 0, 0, 0, 0]),
        0xA0 => return EscapeControl::Append([size + 2, 2, b'\\', b'_', 0, 0, 0, 0]),
        0x2028 => return EscapeControl::Append([size + 2, 2, b'\\', b'L', 0, 0, 0, 0]),
        0x2029 => return EscapeControl::Append([size + 2, 2, b'\\', b'P', 0, 0, 0, 0]),
        _ => {}
    }
    let encode_char = char::from_u32(code_point);
    if let Some(chr) = encode_char {
        let mut ret_bytes = [size + 2, 0, 0, 0, 0, 0, 0, 0];
        let str = chr.encode_utf8(&mut ret_bytes[2..]);
        ret_bytes[1] = str.as_bytes().len() as u8;
        return EscapeControl::Append(ret_bytes);
    }

    EscapeControl::Append(replacement_char)
}
