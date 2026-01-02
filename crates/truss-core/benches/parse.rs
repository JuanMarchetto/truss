use criterion::{criterion_group, criterion_main, Criterion};
use truss_core::TrussEngine;

fn parse_simple_yaml(c: &mut Criterion) {
    let input = include_str!("../../../benchmarks/fixtures/simple.yml");
    let engine = TrussEngine::new();

    c.bench_function("parse_simple_yaml", |b| {
        b.iter(|| {
            engine.analyze(input);
        })
    });
}

fn parse_medium_yaml(c: &mut Criterion) {
    let input = include_str!("../../../benchmarks/fixtures/medium.yml");
    let engine = TrussEngine::new();

    c.bench_function("parse_medium_yaml", |b| {
        b.iter(|| {
            engine.analyze(input);
        })
    });
}

fn parse_complex_static_yaml(c: &mut Criterion) {
    let input = include_str!("../../../benchmarks/fixtures/complex-static.yml");
    let engine = TrussEngine::new();

    c.bench_function("parse_complex_static_yaml", |b| {
        b.iter(|| {
            engine.analyze(input);
        })
    });
}

fn parse_complex_dynamic_yaml(c: &mut Criterion) {
    let input = include_str!("../../../benchmarks/fixtures/complex-dynamic.yml");
    let engine = TrussEngine::new();

    c.bench_function("parse_complex_dynamic_yaml", |b| {
        b.iter(|| {
            engine.analyze(input);
        })
    });
}

criterion_group!(
    benches,
    parse_simple_yaml,
    parse_medium_yaml,
    parse_complex_static_yaml,
    parse_complex_dynamic_yaml
);
criterion_main!(benches);
