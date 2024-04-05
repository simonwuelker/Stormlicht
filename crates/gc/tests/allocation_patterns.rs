use gc::{Gc, Trace};

#[test]
fn linked_list() {
    struct Node {
        next: Option<Gc<Node>>,
    }

    unsafe impl Trace for Node {
        fn trace(&self) {
            self.next.trace();
        }

        fn root(&self) {
            self.next.root();
        }

        fn unroot(&self) {
            self.next.unroot();
        }
    }

    // Allocate a very long linked list and then drop the head (causing
    // all the cells to be collected)
    let mut next = None;
    for _ in 0..100 {
        let new_node = Gc::new(Node { next });
        next = Some(new_node);
    }

    drop(next);
}

#[test]
fn allocate_a_lot_of_unused_stuff() {
    for _ in 0..1000 {
        // This gc can immediately be deallocated again, neat!
        let _ = Gc::new(1);
    }
}
