use compression::brotli;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{fs, io::Read};

const COMPRESSED_FILE: &'static str =
    "../downloads/brotli/testdata/tests/testdata/alice29.txt.compressed";

fn criterion_benchmark(c: &mut Criterion) {
    let mut data = vec![];
    fs::File::open(COMPRESSED_FILE)
        .expect("alice29.txt.compressed not found, did you run download.sh?")
        .read_to_end(&mut data)
        .unwrap();
    c.bench_function("brotli alice29.txt", |b| {
        b.iter(|| brotli::decode(black_box(&data)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
