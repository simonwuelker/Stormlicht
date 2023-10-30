use compression::brotli;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const COMPRESSED_FILE: &[u8] = include_bytes!(concat!(
    env!("DOWNLOAD_DIR"),
    "/brotli/testdata/tests/testdata/alice29.txt.compressed"
));

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("brotli decompress", "alice29.txt"),
        &COMPRESSED_FILE,
        |b, &data| b.iter(|| brotli::decompress(data)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
