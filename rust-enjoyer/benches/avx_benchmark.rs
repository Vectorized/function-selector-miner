#![allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_theta(c: &mut Criterion) {
    c.bench_function("theta", |b| {
        b.iter(|| {
            let mut a = [0u64; 25];
            let mut b = [0u64; 5];

            // Initialize `a` and `b` with some values, e.g., for testing
            for i in 0..25 {
                a[i] = i as u64;
            }

            black_box(rust_enjoyer::theta(&mut a, &mut b));
        })
    });
}

fn bench_compute(c: &mut Criterion) {
    c.bench_function("compute", |b| {
        b.iter(|| {
            let mut s_avx = rust_enjoyer::sponges_avx::SpongesAvx::default();
            black_box(unsafe { s_avx.compute_selectors() });
        })
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
