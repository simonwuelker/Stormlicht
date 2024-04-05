use gc::{Gc, Trace};

struct Foo {
    bar: Gc<i32>,
    baz: Gc<String>,
}

unsafe impl Trace for Foo {
    fn trace(&self) {
        self.bar.trace();
        self.baz.trace();
    }

    fn root(&self) {
        self.bar.root();
        self.baz.root();
    }

    fn unroot(&self) {
        self.bar.unroot();
        self.bar.unroot();
    }
}

#[test]
fn trace_impl() {
    // For immutable access, only a "Gc" is needed
    let bar = Gc::new(0);
    let baz = Gc::new("test".to_string());

    let container = Foo {
        bar: bar.clone(),
        baz: baz.clone(),
    };

    drop(bar);
    drop(container);

    // bar is garbage
    assert_ne!(gc::collect_garbage(), 0);

    // baz remains usable
    assert_eq!(baz.as_str(), "test");
}
