use std::cell::RefCell;
use std::rc::Rc;

use crate::{Gc, Node};

pub fn new_node(gc: &Gc, name: &str) -> Rc<RefCell<Node>> {
    let node = Node::new(name);
    gc.register(&node);
    node
}

pub fn disconnect(parent: &Rc<RefCell<Node>>, target: &Rc<RefCell<Node>>) {
    parent.borrow_mut().children.borrow_mut().retain(|child| {
        child.upgrade().map_or(true, |rc| !Rc::ptr_eq(&rc, target))
    });
}

pub fn assert_gc_count(gc: &Gc, expected: usize, context: &str) {
    let actual = gc.count_objects();
    assert_eq!(
        actual, expected,
        "【{}】: オブジェクト数が一致しません（期待: {}, 実際: {}）",
        context, expected, actual
    );
}

pub fn build_sample_graph(gc: &Gc) -> (Rc<RefCell<Node>>, Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
    let a = new_node(gc, "A");
    let b = new_node(gc, "B");
    let c = new_node(gc, "C");

    Node::add_child(&a, &b);
    Node::add_child(&b, &c);

    (a, b, c)
}
