use criterion::{Criterion, criterion_group, criterion_main};
use rustycog::Machine;

fn bench_insert_1k(c: &mut Criterion) {
    c.bench_function("insert_1k", |b| {
        b.iter(|| {
            let mut machine = Machine::cold(1);
            for i in 0..1_000 {
                machine.insert_cog(move || i);
            }
        });
    });
}

fn bench_insert_10k(c: &mut Criterion) {
    c.bench_function("insert_10k", |b| {
        b.iter(|| {
            let mut machine = Machine::cold(1);
            for i in 0..10_000 {
                machine.insert_cog(move || i);
            }
        });
    });
}

fn bench_insert_100k(c: &mut Criterion) {
    c.bench_function("insert_100k", |b| {
        b.iter(|| {
            let mut machine = Machine::cold(1);
            for i in 0..100_000 {
                machine.insert_cog(move || i);
            }
        });
    });
}

fn bench_insert_1m(c: &mut Criterion) {
    c.bench_function("insert_1m", |b| {
        b.iter(|| {
            let mut machine = Machine::cold(1);
            for i in 0..1_000_000 {
                machine.insert_cog(move || i);
            }
        });
    });
}

criterion_group!(
    insert_benches,
    bench_insert_1k,
    bench_insert_10k,
    bench_insert_100k,
    bench_insert_1m,
);
criterion_main!(insert_benches);
