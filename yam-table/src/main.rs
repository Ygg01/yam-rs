use yam_dark_core::util::{fast_select_low_bits, print_bin_till};

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
    let fin = select_left_input(input, mask, max_size);
    println!("fin:            {} ({fin})", print_bin_till(fin, max_size));
}

fn select_reverse(input: u64, mask: u64, _u: usize) -> u64 {
    fast_select_low_bits(input.reverse_bits(), mask.reverse_bits()).reverse_bits()
}

fn select_left_input(input: u64, mask: u64, max_size: usize) -> u64 {
    let mask = mask & input;
    let m_input = !mask & input;
    let start = input & !(input << 1);
    let end = input & !(input >> 1);
    let m_start = m_input & !(m_input << 1);
    let m_end = m_input & !(m_input >> 1);
    let m_xor = m_start ^ m_end;
    let m_se = m_end - m_start;

    // -------------
    let se = end.wrapping_sub(start);
    let z = se & m_xor;

    println!(
        "input:          {} ({input})",
        print_bin_till(input, max_size)
    );
    println!(
        "mask:           {} ({mask})",
        print_bin_till(mask, max_size)
    );
    println!(
        "mask input:     {} ({m_input})",
        print_bin_till(m_input, max_size)
    );
    println!("--------------------------");
    // println!("start:          {} ({start})", print_bin_till(start, max_size));
    println!("end:            {} ({end})", print_bin_till(end, max_size));
    println!("se:             {} ({se})", print_bin_till(se, max_size));

    println!("--------------------------");
    println!(
        "m_start:        {} ({m_start})",
        print_bin_till(m_start, max_size)
    );
    println!(
        "m_end:          {} ({m_end})",
        print_bin_till(m_end, max_size)
    );
    // println!("m_xor:          {} ({m_xor})", print_bin_till(m_xor, max_size));
    println!(
        "m_se:           {} ({m_se})",
        print_bin_till(m_se, max_size)
    );
    println!("---------------------------");
    let sse = m_se & se;
    println!("sse:            {} ({sse})", print_bin_till(sse, max_size));
    println!("z:              {} ({z})", print_bin_till(z, max_size));

    mask | z
}
