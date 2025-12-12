pub(crate) fn is_blank_or_break(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\r' || c == b'\n'
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
