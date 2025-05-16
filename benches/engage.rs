use criterion::{Criterion, criterion_group, criterion_main};
use rustycog::Machine;

fn test_function() -> f32 {
    let mut x: f32 = 0.0;
    for _ in 0..10 {
        x = x.sqrt().sin().cos().tan();
    }
    x
}

fn bench_engage_1k(c: &mut Criterion) {
    c.bench_function("engage_1k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..1000 {
                machine.insert_cog(move || test_function());
            }
            machine.wait_until_done();
        });
    });
}

fn bench_engage_10k(c: &mut Criterion) {
    c.bench_function("engage_10k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..10_000 {
                machine.insert_cog(move || test_function());
            }
            machine.wait_until_done();
        });
    });
}

fn bench_engage_100k(c: &mut Criterion) {
    c.bench_function("engage_100k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..100_000 {
                machine.insert_cog(move || test_function());
            }
            machine.wait_until_done();
        });
    });
}

fn bench_engage_100k_8_engines(c: &mut Criterion) {
    c.bench_function("engage_100k_8_engines", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(8);
            for _ in 0..100_000 {
                machine.insert_cog(move || test_function());
            }
            machine.wait_until_done();
        });
    });
}

criterion_group!(
    engage_benches,
    bench_engage_1k,
    bench_engage_10k,
    bench_engage_100k,
    bench_engage_100k_8_engines,
);
criterion_main!(engage_benches);
