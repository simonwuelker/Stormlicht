use criterion::{black_box, criterion_group, criterion_main, Criterion};
use url::URL;

fn simple(c: &mut Criterion) {
    let url = "https://example.com/foobar";

    c.bench_function(url, |b| b.iter(|| black_box(url).parse::<URL>()));
}

criterion_group!(benches, simple);
criterion_main!(benches);
