use gc::Gc;

#[test]
fn basic_collection() {
    // Allocate two gc objects
    let x: Gc<usize> = Gc::new(1);
    let y: Gc<usize> = Gc::new(2);

    // Drop the only reference to "x"
    drop(x);

    // There are no more references to "x", so it can be collected
    let freed_memory = gc::collect_garbage();
    assert!(freed_memory != 0);

    // Now there is no more memory to free
    let freed_memory = gc::collect_garbage();
    assert_eq!(freed_memory, 0);

    // Make sure that "y" is still around and wasn't corrupted
    assert_eq!(*y, 2);
}
