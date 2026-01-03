use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use yam_core::Parser;
use yam_test_bench::write_str_from_event;

const TEST_YAML: &str = include_str!("nested.yaml");
const SMALL_OBJECTS_YAML: &str = include_str!("small_objects.yaml");

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(TEST_YAML.len());
    group
        .sample_size(10)
        .throughput(Throughput::Bytes(TEST_YAML.len() as u64));
    group.bench_function("noop", |b| b.iter(noop));
    group.bench_function("saphyr-nested", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(TEST_YAML);
            write_str_from_event(&mut buff, &mut parser, false);
            assert!(!buff.is_empty());
        })
    });
    let mut buff = String::with_capacity(SMALL_OBJECTS_YAML.len());

    group.bench_function("saphyr-small-obj", |b| {
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

criterion_group!(benches, bench);
criterion_main!(benches);
