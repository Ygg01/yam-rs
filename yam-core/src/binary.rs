use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

const BASE64_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Error while decoding base64
#[derive(Debug)]
pub enum DecodeError {
    InvalidByte(u8),
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // TODO handle cases
        Ok(())
    }
}

#[doc(hidden)]
pub fn decode_as_base64(input: &str) -> Result<Vec<u8>, DecodeError> {
    // A minimal base64 decoding implementation for example purposes
    let filtered_in: Vec<u8> = input
        .as_bytes()
        .iter()
        .copied()
        .filter(|byte| *byte != b' ' && *byte != b'\n' && *byte != b'\r')
        .take_while(|byte| *byte != b'=')
        .collect();

    let mut output = Vec::with_capacity(input.len() * 2);

    let mut iter = filtered_in.chunks_exact(4);
    for byte in &mut iter {
        // SAFETY: `chunks_exact` guarantees exactly 4 elements.
        let byte0 = unsafe { *byte.get_unchecked(0) };
        let byte1 = unsafe { *byte.get_unchecked(1) };
        let byte2 = unsafe { *byte.get_unchecked(2) };
        let byte3 = unsafe { *byte.get_unchecked(3) };

        let a = BASE64_CHARSET
            .iter()
            .position(|b| *b == byte0)
            .ok_or(DecodeError::InvalidByte(byte0))?;
        let b = BASE64_CHARSET
            .iter()
            .position(|b| *b == byte1)
            .ok_or(DecodeError::InvalidByte(byte1))?;
        let c = BASE64_CHARSET
            .iter()
            .position(|b| *b == byte2)
            .ok_or(DecodeError::InvalidByte(byte2))?;
        let d = BASE64_CHARSET
            .iter()
            .position(|b| *b == byte3)
            .ok_or(DecodeError::InvalidByte(byte3))?;

        let v0 = ((a << 2) + (b >> 4)) & 0xFF;
        let v1 = ((b << 4) & 0xFF) + (c >> 2);
        let v2 = ((c << 6) & 0xFF) + d;

        output.push(v0 as u8);
        output.push(v1 as u8);
        output.push(v2 as u8);
    }

    match iter.remainder() {
        [a, b, c] => {
            let a = BASE64_CHARSET
                .iter()
                .position(|i| i == a)
                .ok_or(DecodeError::InvalidByte(*a))?;
            let b = BASE64_CHARSET
                .iter()
                .position(|i| i == b)
                .ok_or(DecodeError::InvalidByte(*b))?;
            let c = BASE64_CHARSET
                .iter()
                .position(|i| i == c)
                .ok_or(DecodeError::InvalidByte(*c))?;

            let v0 = ((a << 2) + (b >> 4)) & 0xFF;
            let v1 = ((b << 4) & 0xFF) + (c >> 2);
            let v2 = (c << 6) & 0xFF;

            output.push(v0 as u8);
            output.push(v1 as u8);
            if v2 != 0 {
                output.push(v2 as u8);
            }
        }
        [a, b] => {
            let a = BASE64_CHARSET
                .iter()
                .position(|i| i == a)
                .ok_or(DecodeError::InvalidByte(*a))?;
            let b = BASE64_CHARSET
                .iter()
                .position(|i| i == b)
                .ok_or(DecodeError::InvalidByte(*b))?;

            let v0 = ((a << 2) + (b >> 4)) & 0xFF;
            let v1 = (b << 4) & 0xFF;

            output.push(v0 as u8);
            if v1 != 0 {
                output.push(v1 as u8);
            }
        }
        [a] => {
            let a = BASE64_CHARSET
                .iter()
                .position(|i| i == a)
                .ok_or(DecodeError::InvalidByte(*a))?;

            let v0 = (a << 2) & 0xFF;

            if v0 != 0 {
                output.push(v0 as u8);
            }
        }
        _ => {}
    };

    Ok(output)
}

#[doc(hidden)]
#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn encode_as_base64(input: &[u8]) -> String {
    let mut output = Vec::new();

    let mut iter = input.chunks_exact(3);
    for b in &mut iter {
        // SAFETY: `chunks_exact` guarantees exactly 3 elements.
        let b0 = unsafe { b.get_unchecked(0) };
        let b1 = unsafe { b.get_unchecked(1) };
        let b2 = unsafe { b.get_unchecked(2) };

        // Base 64 encoding based on https://arxiv.org/pdf/1910.05109 pg. 2 heading BASE64
        let i0 = b0 >> 2;
        let i1 = ((b1 >> 4) + (b0 << 4)) & 0x3F;
        let i2 = ((b1 << 2) + (b2 >> 6)) & 0x3F;
        let i3 = b2 & 0x3F;
        output.push(BASE64_CHARSET[i0 as usize]);
        output.push(BASE64_CHARSET[i1 as usize]);
        output.push(BASE64_CHARSET[i2 as usize]);
        output.push(BASE64_CHARSET[i3 as usize]);
    }

    match iter.remainder() {
        [b0] => {
            let i0 = b0 >> 2;
            let i1 = (b0 << 4) & 0x3F;
            output.push(BASE64_CHARSET[i0 as usize]);
            output.push(BASE64_CHARSET[i1 as usize]);
            output.push(b'=');
            output.push(b'=');
        }
        [b0, b1] => {
            let i0 = b0 >> 2;
            let i1 = ((b0 << 4) + (b1 >> 4)) & 0x3F;
            let i2 = (b1 << 2) & 0x3F;
            output.push(BASE64_CHARSET[i0 as usize]);
            output.push(BASE64_CHARSET[i1 as usize]);
            output.push(BASE64_CHARSET[i2 as usize]);
            output.push(b'=');
        }
        _ => {}
    }

    unsafe { String::from_utf8_unchecked(output) }
}
#[cfg(test)]
mod test {
    use crate::binary::{decode_as_base64, encode_as_base64};

    #[test]
    fn test_encode() {
        let x = b"This is complete shite!!!";
        let out = encode_as_base64(x.as_slice());
        // assert_eq!(out, "VGhpcyBpcyBjb21wbGV0ZSBzaGl0ZSE=");

        let decode = decode_as_base64(&out).unwrap();
        assert_eq!(decode, x);
    }
}
