pub(crate) fn is_blank_or_break(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\r' || c == b'\n'
}

pub(crate) fn is_anchor_char(c: u8) -> bool {
    is_yaml_non_space(c) && !is_flow(c) && c != b'\0'
}

pub(crate) fn is_yaml_non_space(c: u8) -> bool {
    !is_blank(c) && !is_break(c)
}

#[inline]
#[must_use]
pub(crate) fn is_word_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'-'
}

#[inline]
#[must_use]
pub(crate) fn is_blank_or_breakz(c: u8) -> bool {
    c == b'\0' || is_blank(c) || is_break(c)
}

/// Check whether the character is a valid URI character.
#[inline]
#[must_use]
pub(crate) fn is_uri_char(c: u8) -> bool {
    is_word_char(c) || b"#;/?:@&=+$,_.!~*\'()[]%".contains(&c)
}

pub(crate) fn is_break(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

pub(crate) fn is_breakz(c: u8) -> bool {
    c == b'\r' || c == b'\n' || c == b'\0'
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
        b'0'..=b'9' => u32::from(c - b'0'),
        b'a'..=b'f' => u32::from(c - b'a') + 10,
        b'A'..=b'F' => u32::from(c - b'A') + 10,
        _ => unreachable!(),
    }
}

pub(crate) fn is_tag_char(chr: u8) -> bool {
    is_uri_char(chr) && !is_flow(chr) && chr != b'!'
}

#[inline]
#[must_use]
pub fn is_alpha(c: u8) -> bool {
    matches!(c, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-')
}

pub(crate) static LO_BYTES: u64 = u64::from_ne_bytes([0x01; 8]);
static HI_BYTES: u64 = u64::from_ne_bytes([0x80; 8]);
/// Returns a word whose lowest set bit is in the first byte of `word` that equals `byte`, or `0`
/// if there is no such byte. Bits above that byte may be set spuriously.
#[inline]
pub(crate) fn eq_byte_mask(word: u64, byte: u8) -> u64 {
    let x = word ^ (LO_BYTES * u64::from(byte));
    x.wrapping_sub(LO_BYTES) & !x & HI_BYTES
}

/// Returns the length of the longest prefix of `input` without a byte for which `is_stop` holds.
///
/// Eight bytes at a time are checked with `stop_mask`, which gets them as a little-endian `u64`.
/// Its result must have its lowest set bit in the first byte for which `is_stop` holds, or be
/// `0` if there is none.
#[inline]
pub(crate) fn len_until(
    input: &[u8],
    stop_mask: impl Fn(u64) -> u64,
    is_stop: impl Fn(u8) -> bool,
) -> usize {
    let (chunks, tail) = input.as_chunks::<8>();
    for (i, chunk) in chunks.iter().enumerate() {
        let mask = stop_mask(u64::from_le_bytes(*chunk));
        if mask != 0 {
            return i * 8 + (mask.trailing_zeros() / 8) as usize;
        }
    }
    let len = input.len() - tail.len();
    let tail_pos = tail.iter().position(|&c| is_stop(c)).unwrap_or(tail.len());
    len + tail_pos
}
