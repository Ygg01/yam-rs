// fn main() -> std::io::Result<()> {
//     let mut x = Vec::with_capacity(256);
//     for i in 0..=u8::MAX {
//         x.push(calculate_indent(i));
//     }
//     let mut output = String::new();
//     for el in x {
//         use std::fmt::Write;
//         writeln!(&mut output, "{:?},", el).expect("Error");
//     }
//     let file = File::create("foo.txt")?;
//     let mut buf_reader = BufWriter::new(file);
//     buf_reader.write_all(output.as_bytes())?;
//     Ok(())
// }

use yam_dark_core::util::print_bin_till;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum Test {
    Value(i32),
    Nothing,
    Float(f64),
}

#[allow(dead_code)]
fn print2(input: u8) {
    println!("\ninput = {:#010b}", input);

    let start_edge = input & !(input << 1);
    println!("se    = {:#010b}", start_edge);

    let end_edge = input & !(input >> 1);
    println!("ee    = {:#010b}", end_edge);

    let see = start_edge | end_edge;
    println!("see   = {:#010b}", see);

    let fin = input ^ start_edge ^ end_edge;
    println!("in    = {:#010b}", fin);

    let x = see * 2 - end_edge;
    println!("xxx   = {:#010b}", x);
}

fn main() {
    // let input = 0b1100_1100;
    // let mask = 0b1010_1010;
    // let input = 1434;
    // let mask = 272;
    // let max_size = 4;
    let input = 0b1111_0000_1110_0000_0110;
    let mask = 0b1000_0010_0000_0000_0100;
    let max_size = 5;
    let fin = select_left_input(input, mask, max_size);
    println!("fin:            {} ({fin})", print_bin_till(fin, max_size));
}

fn select_left_input(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let ones = input & !(input << 1) & !(input >> 1);
    let m_input = !mask & input & !ones;
    let start = m_input & !(m_input << 1);
    let end = m_input & !(m_input >> 1);
    // -------------
    let se = end.wrapping_sub(start);
    let m_se = mask.wrapping_sub(start);
    let carry = (se ^ m_se) & start;
    let z = mask.wrapping_sub(carry) & input;

    println!(
        "input:          {} ({input})",
        print_bin_till(input, max_size)
    );

    println!(
        "mask input:     {} ({m_input})",
        print_bin_till(m_input, max_size)
    );
    println!(
        "mask:           {} ({mask})",
        print_bin_till(mask, max_size)
    );
    println!(
        "start:          {} ({start})",
        print_bin_till(start, max_size)
    );
    println!("end:            {} ({end})", print_bin_till(end, max_size));
    println!("--------------------------");
    println!("se:             {} ({se})", print_bin_till(se, max_size));
    println!(
        "m_se:           {} ({m_se})",
        print_bin_till(m_se, max_size)
    );
    println!(
        "carry:          {} ({carry})",
        print_bin_till(carry, max_size)
    );
    println!("z:              {} ({z})", print_bin_till(z, max_size));
    mask | z
}

//
// #[allow(dead_code)]
// fn find_even_end(bits: u8) -> u8 {
//     let start_edge = bits & !(bits << 1);
//     let end_edge = bits & !(bits >> 1);
//
//     let even_start = start_edge & 0x55;
//     let odd_start = start_edge & 0xAA;
//
//     let even_carry = bits + even_start;
//     let odd_carry = bits + odd_start;
//
//     let even_carry_only = !bits & even_carry;
//     let odd_carry_only = !bits & odd_carry;
//
//     let odd1 = even_carry_only & 0x55;
//     let odd2 = odd_carry_only & 0xAA;
//
//     let end_edge_even = end_edge & 0x55;
//     let end_edge_odd = end_edge & 0xAA;
//
//     let (max, min, part) = if end_edge_even < end_edge_odd {
//         (end_edge_odd, end_edge_even, 0x55)
//     } else {
//         (end_edge_even, end_edge_odd, 0xAA)
//     };
//
//     let edge_sub = (max << 1).saturating_sub(bits) + min;
//
//     let edge_other = (end_edge << 1).saturating_sub(bits) ^ edge_sub;
//
//     odd1 >> 1 | odd2 >> 1 | (edge_sub & part) | (edge_other & !part)
// }
//
// fn find_odd_end(bits: u8) -> u8 {
//     let start_edge = bits & !(bits << 1);
//     let even_start = start_edge & 0x55;
//     let even_carry = bits + even_start;
//     let even_carry_only = !bits & even_carry;
//
//     let odd1 = even_carry_only & 0xAA;
//     let odd_starts = start_edge & 0xAA;
//     let odd_carries = bits.overflowing_add(odd_starts).0;
//     let odd_carries_only = odd_carries & !bits;
//     let odd2 = odd_carries_only & 0x55;
//
//     (odd1 | odd2) >> 1
// }
//
// fn find_odd_start(bits: u8) -> u8 {
//     let end_edge = bits & !(bits >> 1);
//     // println!("eef     = {:#010b}", end_edge);
//
//     let even_end = end_edge & 0x55;
//     let odd_end = end_edge & 0xAA;
//
//     let (max, min, part) = if even_end < odd_end {
//         (odd_end, even_end, 0xAA)
//     } else {
//         (even_end, odd_end, 0x55)
//     };
//
//     let edge_sub = (max << 1).saturating_sub(bits) + min;
//     let edge_other = (end_edge << 1).saturating_sub(bits) ^ edge_sub;
//
//     let odd1 = edge_sub & part;
//     let odd2 = edge_other & !part;
//
//     odd1 | odd2
// }
//
// fn print3(input: u8) {
//     println!("\nin      = {:#010b}", input);
//
//     println!("fos     = {:#010b}", find_odd_start(input));
//     println!("foe     = {:#010b}", find_odd_end(input));
//     println!("fee     = {:#010b}", find_even_end(input));
// }
//
// #[allow(unused)]
// fn scale(xxx: u8) -> u8 {
//     let mut scale = xxx;
//     scale ^= scale << 1;
//     scale ^= scale << 2;
//     scale ^= scale << 4;
//     println!("^^^   = {:#010b}", scale);
//     scale
// }
//
// #[allow(unused)]
// fn calculate_indent(mask: u8) -> [u8; 8] {
//     let mut result = [0, 1, 2, 3, 4, 5, 6, 7];
//     let mut start_pos = None;
//     for (pos, item) in result.iter_mut().enumerate() {
//         if mask & (1 << pos) != 0 && start_pos.is_some() {
//             start_pos = None;
//         }
//         let old_val = *item;
//         *item = start_pos.unwrap_or(old_val);
//         if mask & (1 << pos) == 0 && start_pos.is_none() {
//             start_pos = Some(*item);
//         }
//     }
//     result
// }
