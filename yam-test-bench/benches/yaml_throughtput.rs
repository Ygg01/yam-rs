use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use yam_core::Parser;
use yam_test_bench::write_str_from_event;

const NESTED_YAML: &str = include_str!("nested.yaml");

const BIG_TEXT: &str = include_str!("big_text.yaml");
const SMALL_OBJECTS_YAML: &str = include_str!("small_objects.yaml");

fn bench_noop(c: &mut Criterion) {
    let mut group = c.benchmark_group("noop-bench");
    group.bench_function("noop", |b| b.iter(noop));
    group.finish();
}

fn bench_big_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("big-text");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(BIG_TEXT.len());
    group
        .sample_size(50)
        .throughput(Throughput::Bytes(BIG_TEXT.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(BIG_TEXT);
            write_str_from_event(&mut buff, &mut parser, false);
            assert!(!buff.is_empty());
        })
    });

    group.finish();
}

fn bench_nested(c: &mut Criterion) {
    let mut group = c.benchmark_group("big-nested");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(NESTED_YAML.len());
    group
        .sample_size(50)
        .throughput(Throughput::Bytes(NESTED_YAML.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(NESTED_YAML);
            write_str_from_event(&mut buff, &mut parser, false);
            assert!(!buff.is_empty());
        })
    });

    group.finish();
}

fn bench_small_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("small-object");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(SMALL_OBJECTS_YAML.len());
    group
        .sample_size(50)
        .throughput(Throughput::Bytes(SMALL_OBJECTS_YAML.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(SMALL_OBJECTS_YAML);
            write_str_from_event(&mut buff, &mut parser, false);
            assert!(!buff.is_empty());
        })
    });

    group.finish();
}

fn noop() {}

criterion_group!(
    benches,
    bench_noop,
    bench_big_text,
    bench_nested,
    bench_small_object
);
criterion_main!(benches);
