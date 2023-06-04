extern crate steel_yaml;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use steel_yaml::tokenizer::assert_eq_event;

const IN1: &'static str = r#"
[{:}]
"#;

const IN1_EXPECTED: &'static str = r#"
+DOC
+SEQ []
+MAP {}
=VAL :
=VAL :
-MAP
-SEQ
-DOC"#;

fn bench_str_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.05).sample_size(100);
    group.bench_function("bench_yaml", |b| {
        b.iter(|| assert_eq_event(black_box(IN1), black_box(IN1_EXPECTED)))
    });
    group.finish();
}

criterion_group!(benches, bench_str_iter);
criterion_main!(benches);
