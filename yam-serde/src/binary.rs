use alloc::string::String;
use alloc::vec::Vec;

const BASE64_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn decode_as_base64(input: &str) -> Vec<u8> {
    // A minimal base64 decoding implementation for example purposes
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for &byte in input.as_bytes() {
        if byte == b'=' {
            break;
        } else if byte == b' ' || byte == b'\n' || byte == b'\r' {
            continue;
        }
        if let Some(position) = BASE64_CHARSET.iter().position(|&c| c == byte) {
            buffer = (buffer << 6) | (position as u32);
            bits_collected += 6;
            if bits_collected >= 8 {
                bits_collected -= 8;
                output.push((buffer >> bits_collected) as u8);
            }
        }
    }

    output
}

fn encode_as_base64(input: &[u8]) -> String {
    let mut output = String::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for &byte in input {
        buffer = (buffer << 8) | (byte as u32);
        bits_collected += 8;
        while bits_collected >= 6 {
            bits_collected -= 6;
            let index = (buffer >> bits_collected) as usize & 0x3F;
            output.push(BASE64_CHARSET[index] as char);
        }
    }

    if bits_collected > 0 {
        let index = (buffer << (6 - bits_collected)) as usize & 0x3F;
        output.push(BASE64_CHARSET[index] as char);
        while !output.len().is_multiple_of(4) {
            output.push('=');
        }
    }

    output
}
