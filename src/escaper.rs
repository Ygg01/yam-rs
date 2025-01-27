use std::{borrow::Cow, ops::Deref};

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
                    self.pos =  pos + step as usize;
                    return Some(NewPos { pos, step, len, bytes: [bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]});
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

pub fn escape_plain(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n',
        |input| {
            match input {
                [b'\\', b't', ..] | [b'\\', b'r', ..] | [b'\\', b'n', ..] => EscapeControl::Skip(2),
                [b'\r', ..] => EscapeControl::Append([1, 2, b'\\', b'r', 0, 0, 0, 0]),
                [b'\t', ..] => EscapeControl::Append([1, 2, b'\\', b't', 0, 0, 0, 0]),
                [b'\n', ..] => EscapeControl::Append([1, 2, b'\\', b'n', 0, 0, 0, 0]),
                [b'\\', ..] => EscapeControl::Append([1, 2, b'\\', b'\\', 0, 0, 0, 0]),
                _ => EscapeControl::Break,
            }
        },
    )
}

pub fn escape_double_quotes(input: Cow<'_, [u8]>) -> Cow<'_, [u8]> {
    _escape(
        input,
        |&chr| chr == b'\t' || chr == b'\\' || chr == b'\n' || chr == b'\r',
        |input| {
            match input {
                [b'\\', b't', ..] | [b'\\', b'r', ..] | [b'\\', b'n', ..] => EscapeControl::Skip(2),
                [b'\\', b'\t', ..] => EscapeControl::Append([2, 2, b'\\', b't', 0, 0, 0, 0]),
                [b'\\', b'/', ..] => EscapeControl::Append([2, 1, b'/', 0, 0, 0, 0, 0]),
                [b'\r', ..] => EscapeControl::Append([1, 2, b'\\', b'r', 0, 0, 0, 0]),
                [b'\t', ..] => EscapeControl::Append([1, 2, b'\\', b't', 0, 0, 0, 0]),
                [b'\n', ..] => EscapeControl::Append([1, 2, b'\\', b'n', 0, 0, 0, 0]),
                [b'\\', ..] => EscapeControl::Append([1, 2, b'\\', b'\\', 0, 0, 0, 0]),
                [b'\'', ..] => EscapeControl::Append([1, 2, b'\\', b'\'', 0, 0, 0, 0]),
                _ => EscapeControl::Break,
            }
        },
    )
}

fn _escape<F, M>(input: Cow<[u8]>, find_fn: F, match_fn: M) -> Cow<[u8]>
where
    F: Fn(&u8) -> bool,
    M: Fn(&[u8]) -> EscapeControl,
{
    let raw = input.deref();
    let escape_iter = Escape::new(raw, find_fn, match_fn);
    let mut old_pos = 0;
    let mut escaped: Option<Vec<u8>> = None;
    let mut _cont = true;

    for NewPos {pos, step, len, bytes} in  escape_iter {
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
