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

#[derive(Debug, Copy, Clone)]
enum Test {
    Value(i32),
    Nothing,
    Float(f64),
}
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
    // print3(0b1111);
    // print3(0b1111000);
    // print3(0b0011);
    // print3(0b01111101);
    // print3(0b010111101);
    // print3(0b0111100);
    // print3(0b1111101);
    // let x = 0b10111;
    // print3(x);
    // print3(0b10111);
    print3(0b1111);
    print3(0b11101);
    // print3(0b1011010);
    // print3(0b1011101);
    // print3(0b110);
}

fn find_odd_start(bits: u8) -> u8 {
    let end_edge = bits & !(bits >> 1);
    println!("ee    = {:#010b}", end_edge);
    let start_edge = bits & !(bits << 1);


    let end_edge_odd = (end_edge & 0xAA);
    // println!("eeo   = {:#010b}", end_edge_odd);
    let end_edge_even = (end_edge & 0x55);
    // println!("eee   = {:#010b}", end_edge_even);

    let (max, min, part) = if end_edge_even < end_edge_odd {
        (end_edge_odd, end_edge_even, 0xAA)
    } else {
        (end_edge_even, end_edge_odd, 0x55)
    };

    let edge_sub = (max << 1).saturating_sub(bits) + min;
    println!("es    = {:#010b}", edge_sub);

    let edge_other = (end_edge << 1).saturating_sub(bits) ^ edge_sub;
    println!("eo    = {:#010b}", edge_other);

    // let fin = 0;
    let mut fin = edge_sub & part | edge_other & !part;
    println!("fin   = {:#010b}", fin);

    fin
}

fn find_odd_end(bits: u8) -> u8 {
    let start_edge = bits & !(bits << 1);
    let even_start = start_edge & 0x55;
    let even_carry = bits + even_start;
    let even_carry_only = !bits & even_carry;

    let odd1 = even_carry_only & 0xAA;
    let odd_starts = start_edge & 0xAA;
    let odd_carries = bits.overflowing_add(odd_starts).0;
    let odd_carries_only = odd_carries & !bits;
    let odd2 = odd_carries_only & 0x55;

    (odd1 | odd2) >> 1
}

fn print3(input: u8) {
    println!("\nin    = {:#010b}", input);

    // let sa = input & !(input << 1);
    // let in_wos = input & !sa;
    let odd_start = find_odd_start(input);
    // println!("ods   = {:#010b}", odd_start);

    let odd_end = find_odd_end(input);
    // println!("evs   = {:#010b}", odd_end);

    // let left_pad = in_wos & (in_wos << 1);
    // let right_pad = in_wos & (in_wos >> 1);
    //
    // let inn = left_pad & right_pad;
    // println!("inn   = {:#010b}", inn);
    //
    // // let ee = input & !(input >>1);
    // let out = (left_pad | right_pad);
    // println!("out   = {:#010b}", out);
    //
    // let odd = find_odd(inn);
    // println!("odd   = {:#010b}", odd);
    //
    // let xxx = input ^ odd;
    // println!("xxx   = {:#010b}", xxx);
    //
    // let xxx = scale(xxx);
    //
    // let yyy = input ^ out;
    // println!("yyy   = {:#010b}", xxx);
    // let yyy = scale(yyy);
    //
    // let fin = xxx | yyy;
    // println!("final = {:#010b}", fin);
}

fn scale(xxx: u8) -> u8 {
    let mut scale = xxx;
    scale ^= scale << 1;
    scale ^= scale << 2;
    scale ^= scale << 4;
    println!("^^^   = {:#010b}", scale);
    scale
}

fn calculate_indent(mask: u8) -> [u8; 8] {
    let mut result = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut start_pos = None;
    for (pos, item) in result.iter_mut().enumerate() {
        if mask & (1 << pos) != 0 && start_pos.is_some() {
            start_pos = None;
        }
        let old_val = *item;
        *item = start_pos.unwrap_or(old_val);
        if mask & (1 << pos) == 0 && start_pos.is_none() {
            start_pos = Some(*item);
        }
    }
    result
}
