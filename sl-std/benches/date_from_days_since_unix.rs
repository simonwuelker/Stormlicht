use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sl_std::datetime::Date;

fn criterion_benchmark(c: &mut Criterion) {
    let days = 123456;

    c.bench_with_input(
        BenchmarkId::new("Date from_days_since_unix", days),
        &days,
        |b, &s| b.iter(|| Date::new_from_days_since_unix(s)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
