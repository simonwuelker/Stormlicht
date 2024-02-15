use perfect_hash::perfect_set;

perfect_set!(EXAMPLE_SET, ["foo", "bar", "baz", "foo-bar",]);

#[test]
fn test_static_set() {
    assert!(EXAMPLE_SET.contains("foo"));
    assert!(!EXAMPLE_SET.contains("fooz"));

    let foo_bar = EXAMPLE_SET
        .try_get("foo-bar")
        .expect("Static set doesn't contain foo-bar");
    assert_eq!(foo_bar, static_str!("foo-bar"));
    assert_ne!(foo_bar, static_str!("baz"));
}
