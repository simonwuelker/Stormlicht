use perfect_hash::perfect_set;

perfect_set!(
    const EXAMPLE_SET = [
        "foo",
        "bar",
        "baz",
        "foobar",
    ];
);

fn main() {
    assert!(EXAMPLE_SET.contains("foo"));
    assert!(!EXAMPLE_SET.contains("fooz"));
}
