use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;

use yam_dark_core::{
    ChunkyIterWrap, NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState,
};

const YAML: &str = "
    a: xeirstr
    b: zcxczxc
    c: rteart
    d: u34yuo
    e: 8uypyuwq5k

z
xc
vx
cv
xc
v
xc
v
xc
vvvvvvvvvvv
oevirsntierst
";

fn bench_stage1(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-stage1");
    let mut iter = ChunkyIterWrap::from_bytes(YAML.as_bytes());
    println!("bytes: {:#?}", YAML.as_bytes().len());
    let mut state = YamlParserState::default();

    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(YAML.len() as u64));
    group.bench_function("bench-dark-yam", |b| {
        b.iter(|| {
            for chunk in iter.by_ref() {
                let chunk_state: YamlChunkState = NativeScanner::next(chunk, &mut state, &mut 0);
                state.process_chunk::<NativeScanner>(&chunk_state);
            }
            for chr in iter.remainder() {
                if *chr == b'x' {
                    state.structurals.push(4)
                }
            }
            black_box(!state.structurals.is_empty());
        });
    });
    group.finish();
}

criterion_group!(benches, bench_stage1);
criterion_main!(benches);
