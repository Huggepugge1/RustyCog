use criterion::{Criterion, criterion_group, criterion_main};
use rustycog::{Machine, types::CogId};

fn test_function() -> f32 {
    let mut x: f32 = 0.0;
    for _ in 0..10 {
        x = x.sqrt().sin().cos().tan();
    }
    x
}

fn bench_retrieve_1k(c: &mut Criterion) {
    c.bench_function("retrieve_1k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..1000 {
                machine.insert_cog(move || test_function());
            }
            for i in 0..1000 {
                machine.wait_for_result(i as CogId).unwrap();
            }
        });
    });
}

fn bench_retrieve_10k(c: &mut Criterion) {
    c.bench_function("retrieve_10k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..10_000 {
                machine.insert_cog(move || test_function());
            }
            for i in 0..10_000 {
                let _ = machine.wait_for_result(i as CogId);
            }
        });
    });
}

fn bench_retrieve_10k_8_engines(c: &mut Criterion) {
    c.bench_function("retrieve_10k_8_engies", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(8);
            for _ in 0..10_000 {
                machine.insert_cog(move || test_function());
            }
            for i in 0..10_000 {
                let _ = machine.wait_for_result(i as CogId);
            }
        });
    });
}

fn bench_retrieve_100k(c: &mut Criterion) {
    c.bench_function("retrieve_100k", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(1);
            for _ in 0..100_000 {
                machine.insert_cog(move || test_function());
            }
            for i in 0..100_000 {
                let _ = machine.wait_for_result(i as CogId);
            }
        });
    });
}

fn bench_retrieve_100k_8_engines(c: &mut Criterion) {
    c.bench_function("retrieve_100k_8_engies", |b| {
        b.iter(|| {
            let mut machine = Machine::powered(8);
            for _ in 0..100_000 {
                machine.insert_cog(move || test_function());
            }
            for i in 0..100_000 {
                let _ = machine.wait_for_result(i as CogId);
            }
        });
    });
}

criterion_group!(
    retrieve_benches,
    bench_retrieve_1k,
    bench_retrieve_10k,
    bench_retrieve_10k_8_engines,
    bench_retrieve_100k,
    bench_retrieve_100k_8_engines,
);
criterion_main!(retrieve_benches);
