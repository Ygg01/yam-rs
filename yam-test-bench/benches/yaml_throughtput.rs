use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;
use yam_core::parsing::{BufferedBytesSource, Parser};
use yam_test_bench::{write_str_from_event, FormatOpts};

const NESTED_YAML: &str = include_str!("nested.yaml");
const BIG_TEXT: &str = include_str!("big_text.yaml");
const BIG_QUOTE_TEXT_YAML: &str = include_str!("big_quote_text.yaml");
const SMALL_OBJECTS_YAML: &str = include_str!("small_objects.yaml");

fn bench_noop(c: &mut Criterion) {
    let mut group = c.benchmark_group("noop-bench");
    group.bench_function("noop", |b| b.iter(noop));
    group.finish();
}

const SAMPLE_SIZE: usize = 1000;

fn bench_big_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("big-text");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(BIG_TEXT.len());
    group
        .sample_size(SAMPLE_SIZE)
        // Iterations are short, so `SAMPLE_SIZE` linear samples don't fit into the default 5s.
        .measurement_time(Duration::from_secs(10))
        .throughput(Throughput::Bytes(BIG_TEXT.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(BIG_TEXT);
            write_str_from_event(&mut buff, &mut parser, &FormatOpts::YamlFormat);
            assert!(!buff.is_empty());
        })
    });
    group.bench_function("parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new_from_str(BIG_TEXT) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });
    group.bench_function("buffered-parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new(BufferedBytesSource::new_from_str(BIG_TEXT)) {
                black_box(ev.expect("bench input must parse"));
            }
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
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Bytes(NESTED_YAML.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(NESTED_YAML);
            write_str_from_event(&mut buff, &mut parser, &FormatOpts::YamlFormat);
            assert!(!buff.is_empty());
        })
    });
    group.bench_function("parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new_from_str(NESTED_YAML) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });
    group.bench_function("buffered-parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new(BufferedBytesSource::new_from_str(NESTED_YAML)) {
                black_box(ev.expect("bench input must parse"));
            }
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
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Bytes(SMALL_OBJECTS_YAML.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(SMALL_OBJECTS_YAML);
            write_str_from_event(&mut buff, &mut parser, &FormatOpts::YamlFormat);
            assert!(!buff.is_empty());
        })
    });
    group.bench_function("parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new_from_str(SMALL_OBJECTS_YAML) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });
    group.bench_function("buffered-parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new(BufferedBytesSource::new_from_str(SMALL_OBJECTS_YAML)) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });

    group.finish();
}

fn bench_quoted(c: &mut Criterion) {
    let mut group = c.benchmark_group("big-text-quoted");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    let mut buff = String::with_capacity(BIG_QUOTE_TEXT_YAML.len());
    group
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Bytes(BIG_QUOTE_TEXT_YAML.len() as u64));
    group.bench_function("saphyr", |b| {
        b.iter(|| {
            buff.clear();
            let mut parser = Parser::new_from_str(BIG_QUOTE_TEXT_YAML);
            write_str_from_event(&mut buff, &mut parser, &FormatOpts::YamlFormat);
            assert!(!buff.is_empty());
        })
    });
    group.bench_function("parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new_from_str(BIG_QUOTE_TEXT_YAML) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });
    group.bench_function("buffered-parse-only", |b| {
        b.iter(|| {
            for ev in Parser::new(BufferedBytesSource::new_from_str(BIG_QUOTE_TEXT_YAML)) {
                black_box(ev.expect("bench input must parse"));
            }
        })
    });

    group.finish();
}

fn noop() {}

criterion_group!(
    benches,
    bench_noop,
    bench_big_text,
    bench_quoted,
    bench_nested,
    bench_small_object
);
criterion_main!(benches);
