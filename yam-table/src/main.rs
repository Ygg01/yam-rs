use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let mut x = Vec::with_capacity(256);
    for i in 0..=u8::MAX {
        x.push(calculate_indent(i));
    }
    let mut output = String::new();
    for el in x {
        use std::fmt::Write;
        write!(&mut output, "{:?}\n", el).expect("Error");
    }
    let file = File::create("foo.txt")?;
    let mut buf_reader = BufWriter::new(file);
    buf_reader.write(output.as_bytes())?;
    Ok(())
}

fn calculate_indent(mask: u8) -> [u8; 8] {
    let mut result = [0; 8];
    let mut last_index = 0usize;
    for pos in 0u8..8 {
        if mask & (1 << pos) == 0 {
            result[last_index] = pos;
            last_index += 1;
        }
    }
    result
}
