use criterion::{black_box,Criterion, criterion_group, criterion_main, Throughput};
use yam_dark_core::ChunkyIterator;

const BYTE: [u8; 128] = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

fn chunky_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(BYTE.len() as u64));
    group.bench_function("bench_chunky",  |b| {
        b.iter(|| ChunkyIterator::from_bytes(&BYTE).filter(|x| x[0] > 5));
    });
    group.finish();
}

fn bytes_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(BYTE.len() as u64));
    group.bench_function("bytes_iter", |b| {
        b.iter(|| BYTE.iter().filter(|x| **x > 5));
    });
    group.finish();
}

criterion_group!(benches, chunky_iter, bytes_iter);
criterion_main!(benches);