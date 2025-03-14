use yam_dark_core::util::print_bin_till;

fn main() {
    let input = 0b1111_0000_0110_0000_0110;
    let maska = 0b1001_0000_0000_0000_0100;
    let max_size = 5;

    // let input = 0b1_1111;
    // let maska = 0b0_1010;
    // let max_size = 2;

    let mask = maska & input;
    println!("input:     {} ({input})", print_bin_till(input, max_size));
    println!("mask:      {} ({mask})", print_bin_till(mask, max_size));

    let fin = select_left_input4(input, mask, max_size);
    println!("fin:       {} ({fin})", print_bin_till(fin, max_size));
}

fn select_left_input4(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let start = input & !(input << 1);
    // DO NOT TOUCH
    let hm = input.wrapping_add(mask) & mask;

    let start_fin = 0;
    println!("--------------------------");
    println!("hm:        {} ({hm})", print_bin_till(hm, max_size));

    println!(
        "sf:        {} ({start_fin})",
        print_bin_till(start_fin, max_size)
    );

    start_fin | mask
}

fn select_left_input3(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let start = input & !(input << 1);
    println!("start:     {} ({start})", print_bin_till(start, max_size));

    println!("--------------------------");

    let mask = find_greatest_mask(input, mask, max_size);

    println!("mask':     {} ({mask})", print_bin_till(mask, max_size));
    let diff = mask.wrapping_sub(input) & input;
    let diff2 = mask.abs_diff(start);
    let carried = diff & !(diff >> 1);

    let x = mask.wrapping_sub(diff);

    let ms = (diff2 + carried) & start;
    let md = mask.saturating_sub(ms) & input;
    println!("diff:      {} ({diff})", print_bin_till(diff, max_size));
    println!("x:         {} ({x})", print_bin_till(x, max_size));
    println!("diff2:     {} ({diff2})", print_bin_till(diff2, max_size));

    println!(
        "carried:   {} ({carried})",
        print_bin_till(carried, max_size)
    );

    // println!("carry:     {} ({carry})", print_bin_till(carry, max_size));
    println!("ms:        {} ({ms})", print_bin_till(ms, max_size));
    println!("md:        {} ({md})", print_bin_till(md, max_size));

    input & (md | mask)
}

fn find_greatest_mask(input: u64, mask: u64, max_size: usize) -> u64 {
    let im = mask.wrapping_add(input);
    let imc = !(im ^ input) & mask;
    let dupm = im.saturating_sub(imc + mask);
    println!("im:        {} ({im})", print_bin_till(im, max_size));
    println!("imc:       {} ({imc})", print_bin_till(imc, max_size));
    println!("dupm:      {} ({dupm})", print_bin_till(dupm, max_size));
    println!("--------------------------");

    mask & !dupm
}

fn select_left_input2(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let start = input & !(input << 1);

    println!("--------------------------");

    let diff = mask.abs_diff(start);
    let carried = diff ^ start;
    let carry = carried & !(carried << 1);
    // works excellent except in presence of multiple masks on same part
    let ms = (diff + (carry >> 1)) & start;
    let md = mask.saturating_sub(ms) & input;
    println!("diff:      {} ({diff})", print_bin_till(diff, max_size));
    println!("carry:     {} ({carry})", print_bin_till(carry, max_size));
    println!("ms:        {} ({ms})", print_bin_till(ms, max_size));
    println!("md:        {} ({md})", print_bin_till(md, max_size));

    input & (mask | md)
}

fn select_left_input(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let start = input & !(input << 1);

    println!("start:     {} ({start})", print_bin_till(start, max_size));

    println!("--------------------------");

    let diff = mask.wrapping_sub(start);
    let carry = diff & !input;
    let carried = carry & !(carry >> 1);
    let diffs = diff.wrapping_add(carried);

    let ms = diffs.wrapping_sub(mask) & start;
    let md = mask.saturating_sub(ms);
    println!("diff:      {} ({diff})", print_bin_till(diff, max_size));
    println!("carry:     {} ({carry})", print_bin_till(carry, max_size));
    println!(
        "carried:   {} ({carried})",
        print_bin_till(carried, max_size)
    );

    println!("diffs:     {} ({diffs})", print_bin_till(diffs, max_size));
    println!("ms:        {} ({ms})", print_bin_till(ms, max_size));
    println!("md:        {} ({md})", print_bin_till(md, max_size));
    md | mask
}
