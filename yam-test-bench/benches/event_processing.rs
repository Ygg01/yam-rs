extern crate yam_core;

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_core::tokenizer::assert_eq_event;

const IN1: &str = r#"
&seq
[[name        , hr, avg  ],
[Mark McGwire, 65, 0.278],
[Sammy Sosa  , 63, 0.288]]
"#;

const IN1_EXPECTED: &str = r#"
+DOC
+SEQ [] &seq
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


const FOLD_INPUT: &str = r"
--- >
 Sammy Sosa completed another
 fine season with great stats.

   63 Home Runs
   0.288 Batting Average

 What a year!";

const FOLD_EXPECTED: &str = r"
+DOC ---
=VAL >Sammy Sosa completed another fine season with great stats.\n\n  63 Home Runs\n  0.288 Batting Average\n\nWhat a year!\n
-DOC";

fn bench_folded(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.throughput(Throughput::Bytes(FOLD_INPUT.as_bytes().len() as u64));
    group.bench_function("bench_plain", |b| {
        b.iter(|| assert_eq_event(black_box(FOLD_INPUT), black_box(FOLD_EXPECTED)));
    });
    group.finish();
}

fn bench_flow_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.throughput(Throughput::Bytes(IN1.as_bytes().len() as u64));
    group.bench_function("bench_flow_simple", |b| {
        b.iter(|| assert_eq_event(black_box(IN1), black_box(IN1_EXPECTED)));
    });
    group.finish();
}

fn bench_block_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-yaml");
    group.throughput(Throughput::Bytes(IN2.as_bytes().len() as u64));
    group.bench_function("bench_block_simple", |b| {
        b.iter(|| assert_eq_event(black_box(IN2), black_box(IN2_EXPECTED)));
    });
    group.finish();
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.01).sample_size(500).warm_up_time(Duration::from_millis(10));
    targets = bench_flow_simple, bench_block_simple, bench_folded
}
criterion_main!(benches);
