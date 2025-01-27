use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::ChunkyIterator;

const YAML: &[u8] = r#"
   a: b
   c: b

   d: a   as
  "#
.as_bytes();

const LEN: u64 = YAML.len() as u64;

fn col_count_naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-cols");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(LEN));
    group.bench_function("col_naive", |b| {
        b.iter(|| ChunkyIterator::from_bytes(YAML).filter(|x| x[0] > 5));
    });
    group.finish();
}

criterion_group!(benches, col_count_naive);
criterion_main!(benches);
