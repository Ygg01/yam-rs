use core::fmt::{Error, Write};

pub(crate) fn peekz_byte(array: &[u8], pos: usize) -> u8 {
    if pos < array.len() { array[pos] } else { 0 }
}
pub(crate) fn decode_hex<W: Write>(writer: &mut W, digit_slice: &[u8]) -> Result<(), Error> {
    if !digit_slice.iter().all(u8::is_ascii_hexdigit) {
        writer.write_char('\u{FFFD}')?;
        return Ok(());
    }

    let code_point = digit_slice
        .iter()
        .map(|x| match *x {
            n @ b'0'..=b'9' => n - b'0',
            a @ b'a'..=b'f' => a - b'a' + 10,
            a @ b'A'..=b'F' => a - b'A' + 10,
            _ => 0u8,
        })
        .fold(0u32, |acc, digit| (acc << 4) + u32::from(digit));
    match code_point {
        // YAML has special escape rules for certain values
        // See more in https://yaml.org/spec/1.2.2/#57-escaped-characters
        0 => writer.write_char('\u{FFFD}')?,
        0x07 => writer.write_str("\\a")?,
        0x08 => writer.write_str("\\b")?,
        0x09 => writer.write_str("\\t")?,
        0x0A => writer.write_str("\\n")?,
        0x0B => writer.write_str("\\v")?,
        0x0C => writer.write_str("\\f")?,
        0x0D => writer.write_str("\\r")?,
        0x1B => writer.write_str("\\e")?,
        0x22 => writer.write_str("\\\"")?,
        0x2F => writer.write_str("\\/")?,
        0x5C => writer.write_str("\\\\")?,
        0x85 => writer.write_str("\\N")?,
        0xA0 => writer.write_str("\\_")?,
        0x2028 => writer.write_str("\\L")?,
        0x2029 => writer.write_str("\\P")?,
        _ => {
            let encode_char = char::from_u32(code_point);
            if let Some(encode_char) = encode_char {
                return writer.write_char(encode_char);
            }
        }
    }

    Ok(())
}

pub(crate) fn escape_double_quotes<W: Write>(writer: &mut W, value: &str) -> Result<(), Error> {
    let bytes = value.as_bytes();

    let (mut old_pos, mut pos) = (0, 0);
    while pos < bytes.len() {
        let byte_char = bytes[pos];
        let peek_char = peekz_byte(bytes, pos + 1);
        match (byte_char, peek_char) {
            (b'\\', b't' | b'r' | b'n') => {
                // TODO normalize `\r\n` into `\n`
                pos += 2;
            }

            (b'\t', _) => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\t")?;
                pos += 1;
                old_pos = pos;
            }
            (b'\r', b'\n') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\n")?;
                pos += 2;
                old_pos = pos;
            }
            (b'\n', ..) => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                write!(writer, "\\n")?;
                pos += 1;
                old_pos = pos;
            }
            (b'\\', b'x') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 4])?;
                pos += 4;
                old_pos = pos;
            }
            (b'\\', b'u') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 6])?;
                pos += 6;
                old_pos = pos;
            }
            (b'\\', b'U') => {
                let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
                writer.write_str(prev_str)?;
                decode_hex(writer, &bytes[pos + 2..pos + 8])?;
                pos += 8;
                old_pos = pos;
            }
            _ => {
                pos += 1;
            }
        }
    }
    let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
    writer.write_str(prev_str)?;
    Ok(())
}

// TODO Enable or delete
// pub(crate) fn escape_single_quotes<W: Write>(writer: &mut W, value: &str) -> Result<(), Error> {
//     let bytes = value.as_bytes();
//
//     let (mut old_pos, mut pos) = (0, 0);
//     while pos < bytes.len() {
//         let byte_char = bytes[pos];
//         match byte_char {
//             b'\'' => {
//                 let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
//                 writer.write_str(prev_str)?;
//                 write!(writer, "\\'")?;
//                 pos += 1;
//                 old_pos = pos;
//             }
//             _ => {
//                 pos += 1;
//             }
//         }
//     }
//     let prev_str = unsafe { core::str::from_utf8_unchecked(&bytes[old_pos..pos]) };
//     writer.write_str(prev_str)?;
//     Ok(())
// }
