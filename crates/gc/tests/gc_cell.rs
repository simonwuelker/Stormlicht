use gc::{Gc, GcCell};

#[test]
fn gc_cell() {
    // For mutable access, the value must be wrapped in a GcCell
    let bar = Gc::new(1);
    let container = Gc::new(GcCell::new((2, bar)));

    // Lets swap out container.1
    let bar2 = Gc::new(3);
    container.borrow_mut().1 = bar2;

    // bar is garbage
    assert_ne!(gc::collect_garbage(), 0);

    assert_eq!(*container.borrow().1, 3);
}
