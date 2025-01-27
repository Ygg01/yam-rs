extern crate steel_yaml;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use steel_yaml::tokenizer::EventIterator;

const IN1: &'static str = r#"
[{:}]
"#;

const IN1_EXPECTED: &'static str = r#"
 +SEQ
  +MAP
  -MAP
 -SEQ"#;

fn bench_yaml(input_yaml: &str, expect: &str) {
    let mut event = String::new();
    let scan = EventIterator::new_from_string(input_yaml);
    scan.for_each(|x| event.push_str(x.as_ref()));
    assert_eq!(expect, event);
}

fn bench_str_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.1).sample_size(50);
    group.bench_function("bench_yaml", |b| {
        b.iter(|| bench_yaml(black_box(IN1), black_box(IN1_EXPECTED)))
    });
    group.finish();
}

criterion_group!(benches, bench_str_iter);
criterion_main!(benches);
