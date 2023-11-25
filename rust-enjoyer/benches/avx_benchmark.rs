use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_enjoyer::sponges_avx::SpongeComputeSlice;

fn bench_theta(c: &mut Criterion) {
    c.bench_function("theta", |b| {
        b.iter(|| {
            let mut a = [SpongeComputeSlice::default(); 25];
            let mut b = [SpongeComputeSlice::default(); 5];

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

criterion_group!(benches, bench_theta, bench_compute);
criterion_main!(benches);
