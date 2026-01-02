use criterion::{criterion_group, criterion_main, Criterion};

fn parse_large_yaml(c: &mut Criterion) {
    let input = include_str!("../../../benchmarks/fixtures/large.yml");

    c.bench_function("parse_large_yaml", |b| {
        b.iter(|| {
            truss_core::parse(input).expect("parse failed");
        })
    });
}

criterion_group!(benches, parse_large_yaml);
criterion_main!(benches);
