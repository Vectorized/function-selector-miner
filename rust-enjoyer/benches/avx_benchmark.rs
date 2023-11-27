use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_enjoyer::{sponges_avx::SpongeComputeSlice, SmallString};

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
    let function_name = SmallString::new("foo");
    let function_params = SmallString::new("foo");

    c.bench_function("compute", |b| {
        b.iter(|| {
            let mut s_avx = unsafe {
                rust_enjoyer::sponges_avx::SpongesAvx::new(&function_name, 0, &function_params)
            };
            black_box(unsafe { s_avx.compute_selectors() });
        })
    });
}

criterion_group!(benches, bench_theta, bench_compute);
criterion_main!(benches);
