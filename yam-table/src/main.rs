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
        writeln!(&mut output, "{:?},", el).expect("Error");
    }
    let file = File::create("foo.txt")?;
    let mut buf_reader = BufWriter::new(file);
    buf_reader.write_all(output.as_bytes())?;
    Ok(())
}

fn calculate_indent(mask: u8) -> [u8; 8] {
    let mut result = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut start_pos = None;
    for pos in 0usize..8 {
        if mask & (1 << pos) != 0 && start_pos.is_some() {
            start_pos = None;
        }
        let old_val = result[pos];
        result[pos] = start_pos.unwrap_or(old_val);
        if mask & (1 << pos) == 0 && start_pos.is_none() {
            start_pos = Some(result[pos]);
        }
    }
    result
}
