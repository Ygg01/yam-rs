use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{mask_merge, U8X16, VAL};
use yam_dark_core::ChunkyIterator;

const YAML: &[u8] = r#"
   a: b                      
   c: b       


   d: a   as
                  
                  
                  
  "#
.as_bytes();

fn count_subslice(sublice: &mut U8X16) {
    let mut shift_mask = sublice.comp_all(b'\n').to_bitmask();
    shift_mask = !(shift_mask << 1);
    
    let shift = sublice.shift_right(1);
    *sublice = shift.mask_value(shift_mask);

    let shift1 = sublice.shift_right(1).mask_value(shift_mask);
    *sublice += shift1;

    shift_mask &= shift_mask << 1;
    let shift2 = sublice.shift_right(2).mask_value(shift_mask);
    *sublice += shift2;

    shift_mask &= shift_mask << 2;
    let shift4 = sublice.shift_right(4).mask_value(shift_mask);
    *sublice += shift4;

    shift_mask &= shift_mask << 4;
    let shift8 = sublice.shift_right(8).mask_value(shift_mask);
    *sublice += shift8;
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

pub fn count_table(chunk: &[u8; 64]) -> [u32; 64] {
    let v0 = unsafe { U8X16::from_slice(&chunk[0..16]) };
    let v1 = unsafe { U8X16::from_slice(&chunk[16..32]) };
    let v2 = unsafe { U8X16::from_slice(&chunk[32..48]) };
    let v3 = unsafe { U8X16::from_slice(&chunk[48..64]) };

    let t0 = U8X16::from_array(VAL[v0.to_bitmask() as usize]);
    let t1 = U8X16::from_array(VAL[v1.to_bitmask() as usize]);
    let t2 = U8X16::from_array(VAL[v2.to_bitmask() as usize]);
    let t3 = U8X16::from_array(VAL[v3.to_bitmask() as usize]);


    mask_merge(t0, t1, t2, t3)
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

fn col_count_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();

    group.bench_function("col_memo", |b| {
        b.iter(|| {
            let count = count_table(chunk);
            black_box(count[0] == 0);
        })
    });
    group.finish();
}

criterion_group!(benches, col_count_naive, col_count_table);
criterion_main!(benches);
