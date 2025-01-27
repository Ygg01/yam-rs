use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{mask_and_add_u8x16, mask_merge, U8X16};
use yam_dark_core::ChunkyIterator;

const YAML: &[u8] = r#"
   a: b                      
   c: b       


   d: a   as
                  
                  
                  
  "#
.as_bytes();

fn count_subslice(sublice: &mut U8X16) {
    let mask = sublice.comp_all(b'\n').to_bitmask();
    mask_and_add_u8x16(sublice, *sublice >> 1, mask);
    mask_and_add_u8x16(sublice, *sublice >> 2, mask >> 1);
    mask_and_add_u8x16(sublice, *sublice >> 4, mask >> 2);
    mask_and_add_u8x16(sublice, *sublice >> 8, mask >> 4);
}

pub fn count_cols(chunk: &[u8; 64]) -> [u32; 64] {
    let mut v0 = unsafe { U8X16::from_slice(&chunk[0..16]) };
    let mut v1 = unsafe { U8X16::from_slice(&chunk[16..32]) };
    let mut v2 = unsafe { U8X16::from_slice(&chunk[32..48]) };
    let mut v3 = unsafe { U8X16::from_slice(&chunk[48..64]) };

    count_subslice(&mut v0);
    count_subslice(&mut v1);
    count_subslice(&mut v2);
    count_subslice(&mut v3);

    mask_merge(v0, v1, v2, v3)
}

fn col_count_naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();

    group.bench_function("col_naive", |b| {
        b.iter(|| {
            let count = count_cols(chunk);
            black_box(count[0] == 0);
        })
    });
    group.finish();
}

criterion_group!(benches, col_count_naive);
criterion_main!(benches);
