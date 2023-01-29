use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hash::CRC32;
use std::{fs, io::Read};

fn criterion_benchmark(c: &mut Criterion) {
    let mut data = vec![0; 0x8000000];
    fs::File::open("/dev/urandom")
        .unwrap()
        .read_exact(&mut data)
        .unwrap();

    c.bench_function("crc32 128MB", |b| {
        b.iter(|| {
            let mut hasher = CRC32::default();
            hasher.write(black_box(&data));
            black_box(hasher.finish());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
