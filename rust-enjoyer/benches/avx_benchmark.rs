use criterion::{criterion_group, criterion_main, Criterion};
use rust_enjoyer::theta_avx2;

fn bench_theta_avx2(c: &mut Criterion) {
    c.bench_function("theta_avx2", |b| {
        b.iter(|| {
            let mut a = [0u64; 25];
            let mut b = [0u64; 5];

            // Initialize `a` and `b` with some values, e.g., for testing
            for i in 0..25 {
                a[i] = i as u64;
            }

            theta_avx2(&mut a, &mut b);
        })
    });
}

fn bench_theta(c: &mut Criterion) {
    c.bench_function("theta", |b| {
        b.iter(|| {
            let mut a = [0u64; 25];
            let mut b = [0u64; 5];

            // Initialize `a` and `b` with some values, e.g., for testing
            for i in 0..25 {
                a[i] = i as u64;
            }

            rust_enjoyer::theta(&mut a, &mut b);
        })
    });
}

criterion_group!(benches, bench_theta_avx2, bench_theta);
criterion_main!(benches);
