use yam_dark_core::util::{print_bin_till, select_high_bits};

fn main() {
    // let input = 0b1100_1100;
    // let mask = 0b1010_1010;
    // let input = 1434;
    // let mask = 272;
    // let max_size = 4;
    let input = 0b1111_0000_1110_0000_0110;
    let mask = 0b1000_0010_0000_0000_0100;
    let max_size = 5;
    // let input = 0b0111_1110;
    // let mask = 0b0100_1000;
    // let max_size = 2;
    let fin = select_reverse(input, mask, max_size);
    println!("fin:            {} ({fin})", print_bin_till(fin, max_size));
}

fn select_reverse(input: u64, mask: u64, _u: usize) -> u64 {
    select_high_bits(input.reverse_bits(), mask.reverse_bits()).reverse_bits()
}
/*
fn select_left_input(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let start = input & !(input << 1);

    println!(
        "input:          {} ({input})",
        print_bin_till(input, max_size)
    );
    println!(
        "mask:           {} ({mask})",
        print_bin_till(mask, max_size)
    );
    println!(
        "start:          {} ({start})",
        print_bin_till(start, max_size)
    );

    println!("--------------------------");

    let im = input.wrapping_add(mask);
    let is = input.wrapping_add(start);
    let imx = im ^ input;
    let imxs = imx.wrapping_sub(is);
    println!("im:             {} ({im})", print_bin_till(im, max_size));
    println!("imx:            {} ({imx})", print_bin_till(imx, max_size));
    println!("is:             {} ({is})", print_bin_till(is, max_size));
    println!(
        "imxs:           {} ({imxs})",
        print_bin_till(imxs, max_size)
    );

    println!("--------------------------");
    let ime = im & !(im >> 1);
    let imes = ime.wrapping_sub(start);
    let imex = ime & start;

    println!("ime:            {} ({ime})", print_bin_till(ime, max_size));
    println!(
        "imes:           {} ({imes})",
        print_bin_till(imes, max_size)
    );
    println!(
        "imex:           {} ({imex})",
        print_bin_till(imex, max_size)
    );

    mask
}*/
