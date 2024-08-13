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

fn print1(input: u8) {
    println!("\nin  = {:#010b}", input);

    let start_edge = input & !(input << 1);
    // println!("se  = {:#010b}", start_edge);

    let start_edge2 = start_edge << 1;
    // println!("se2 = {:#010b}", start_edge2);
    //
    let start_edge3 = start_edge2 ^ input;
    // println!("se3 = {:#010b}", start_edge3);

    let start_edge4 = start_edge3 & input;
    // println!("se4 = {:#010b}", start_edge4);

    let end_edge = input & !(input >> 1);
    // println!("ee  = {:#010b}", end_edge);

    let end_edge2 = end_edge >> 1;
    // println!("ee2 = {:#010b}", end_edge2);

    let end_edge3 = end_edge2 ^ input;
    // println!("ee3 = {:#010b}", end_edge3);

    let end_edge4 = end_edge3 & input;
    // println!("ee4 = {:#010b}", end_edge4);

    println!("fin = {:#010b}", end_edge4 & start_edge4);
}

fn main() {
    // print3(0b1111);
    // print3(0b1111000);
    // print3(0b0011);
    // print3(0b01111101);
    print3(0b010111101);
    print3(0b0111100);
    print3(0b01100);
}

fn find_odd(bits: u8) -> u8 {
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

    let start_edge = input & (input << 1);
    let end_edge = input & (input >> 1);

    let inn = start_edge & end_edge;
    println!("inn   = {:#010b}", inn);

    let out = start_edge | end_edge;
    println!("out   = {:#010b}", out);

    let odd = find_odd(inn) ^ inn;
    println!("odd   = {:#010b}", odd);

    let mut xxx = input ^ odd;
    println!("xxx   = {:#010b}", xxx);

    xxx ^= xxx << 1;
    xxx ^= xxx << 2;
    xxx ^= xxx << 4;
    println!("xx2   = {:#010b}", xxx);

    let mut yyy = input ^ out;

    yyy ^= yyy << 1;
    yyy ^= yyy << 2;
    yyy ^= yyy << 4;
    println!("yy2   = {:#010b}", yyy);

    println!("final = {:#010b}", xxx | yyy);
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
