pub(crate) fn is_blank_or_break(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\r' || c == b'\n'
}

pub(crate) fn is_anchor_char(c: u8) -> bool {
    is_yaml_non_space(c) && !is_flow(c)
}

pub(crate) fn is_yaml_non_space(c: u8) -> bool {
    !is_blank(c)
}

pub(crate) fn is_break(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

pub(crate) fn is_blank(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

pub(crate) fn is_flow(c: u8) -> bool {
    matches!(c, b',' | b'[' | b']' | b'{' | b'}')
}

#[inline]
#[must_use]
pub fn as_hex(c: u8) -> u32 {
    match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'a'..=b'f' => (c - b'a') as u32 + 10,
        b'A'..=b'F' => (c - b'A') as u32 + 10,
        _ => unreachable!(),
    }
}

#[inline]
#[must_use]
pub fn is_alpha(c: u8) -> bool {
    matches!(c, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-')
}
