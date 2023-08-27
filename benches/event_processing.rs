extern crate steel_yaml;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use steel_yaml::tokenizer::assert_eq_event;

const IN1: &str = r#"
[&map {a: b}]
"#;

const IN1_EXPECTED: &str = r#"
+DOC
+SEQ []
+MAP {} &map
=VAL :a
=VAL :b
-MAP
-SEQ
-DOC"#;

const IN2: &str = r"
&seq
- [name        , hr, avg  ]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa  , 63, 0.288]
";

const IN2_EXPECTED: &str = r"
+DOC
+SEQ &seq
+SEQ []
=VAL :name
=VAL :hr
=VAL :avg
-SEQ
+SEQ []
=VAL :Mark McGwire
=VAL :65
=VAL :0.278
-SEQ
+SEQ []
=VAL :Sammy Sosa
=VAL :63
=VAL :0.288
-SEQ
-SEQ
-DOC";

fn bench_flow_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.05).sample_size(100);
    group.bench_function("bench_flow_simple", |b| {
        b.iter(|| assert_eq_event(black_box(IN1), black_box(IN1_EXPECTED)));
    });
    group.finish();
}

fn bench_block_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.significance_level(0.05).sample_size(100);
    group.bench_function("bench_block_simple", |b| {
        b.iter(|| assert_eq_event(black_box(IN2), black_box(IN2_EXPECTED)));
    });
    group.finish();
}

criterion_group!(benches, bench_flow_simple, bench_block_simple);
criterion_main!(benches);
