use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::random;

#[doc(hidden)]
pub fn select_consecutive_bits(input: u64, selector: u64) -> u64 {
    let mut pos = 0;
    let mut result = 0u64;
    let mut selector = selector;
    loop {
        if selector == 0 || pos > 63 {
            break
        }
        result |= input & selector;
        selector = input & selector >> 1;
        pos += 1;
    }
    result
}

// #[doc(hidden)]
// pub fn select_consecutive_bits_branchless(bits: u64, mask: u64) -> u64 {
//     let mut pos = 0;
//     let mut result = 0u64;
//     let mut mask = mask;
//     loop {
//         if mask == 0 || pos > 63 {
//             break
//         }
//         result |= bits & mask;
//         mask = bits & mask >> 1;
//         pos += 1;
//     }
//     result
// }

fn find_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-quotes");
    let selector: u64 = 0b00000000_00000000_00000000_00000000_00100000_00000000_00000000_00100000;

    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));
    group.bench_function("quotes_branching", |b| {
        b.iter(|| {
            black_box(select_consecutive_bits(random(), selector));
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    find_bits,
);
criterion_main!(benches);
