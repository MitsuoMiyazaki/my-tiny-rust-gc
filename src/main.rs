use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashSet;

struct Node {
    name: String,
    children: RefCell<Vec<Weak<RefCell<Node>>>>,
}

impl Node {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            name: name.to_string(),
            children: RefCell::new(Vec::new()),
        }))
    }

    fn add_child(self_rc: &Rc<RefCell<Node>>, child: &Rc<RefCell<Node>>) {
        self_rc.borrow_mut().children.borrow_mut().push(Rc::downgrade(child));
    }
}

struct Gc {
    objects: RefCell<Vec<Weak<RefCell<Node>>>>,
}

impl Gc {
    fn new() -> Self {
        Gc {
            objects: RefCell::new(Vec::new()),
        }
    }

    fn register(&self, obj: &Rc<RefCell<Node>>) {
        self.objects.borrow_mut().push(Rc::downgrade(obj));
    }

    fn mark(&self, root: &Rc<RefCell<Node>>, marked: &mut HashSet<*const RefCell<Node>>) {
        let mut stack = vec![Rc::clone(root)];
        while let Some(node) = stack.pop() {
            let ptr = Rc::as_ptr(&node);
            if !marked.insert(ptr) {
                continue;
            }

            for weak_child in node.borrow().children.borrow().iter() {
                if let Some(child) = weak_child.upgrade() {
                    stack.push(child);
                }
            }
        }
    }

    fn sweep(&self, marked: &HashSet<*const RefCell<Node>>) {
        let mut objects = self.objects.borrow_mut();
        objects.retain(|weak_node| {
            if let Some(node) = weak_node.upgrade() {
                let ptr = Rc::as_ptr(&node);
                marked.contains(&ptr)
            } else {
                false
            }
        });
    }

    fn collect_garbage(&self, roots: &[Rc<RefCell<Node>>]) {
        let mut marked = HashSet::new();
        for root in roots {
            self.mark(root, &mut marked);
        }
        self.sweep(&marked);
    }

    fn count_objects(&self) -> usize {
        self.objects.borrow().len()
    }
}

fn main() {
    let gc = Gc::new();

    let a = Node::new("A");
    let b = Node::new("B");
    let c = Node::new("C");
    let d = Node::new("D");

    gc.register(&a);
    gc.register(&b);
    gc.register(&c);
    gc.register(&d);

    Node::add_child(&a, &b);
    Node::add_child(&a, &c);
    Node::add_child(&b, &d);

    println!("GC前: {} 個のオブジェクトが登録されている", gc.count_objects());

    gc.collect_garbage(&[a.clone()]);
    println!("GC後: {} 個のオブジェクトが登録されている", gc.count_objects());

    a.borrow_mut().children.borrow_mut().retain(|child| {
        if let Some(child_rc) = child.upgrade() {
            !Rc::ptr_eq(&child_rc, &c)
        } else {
            true
        }
    });

    b.borrow_mut().children.borrow_mut().retain(|child| {
        if let Some(child_rc) = child.upgrade() {
            !Rc::ptr_eq(&child_rc, &c)
        } else {
            true
        }
    });

    gc.collect_garbage(&[a.clone()]);
    println!("C削除後: {} 個のオブジェクトが登録されている", gc.count_objects());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_collects_unreachable_nodes() {
        let gc = Gc::new();

        let a = Node::new("A");
        let b = Node::new("B");
        let c = Node::new("C");

        gc.register(&a);
        gc.register(&b);
        gc.register(&c);

        Node::add_child(&a, &b);
        Node::add_child(&a, &c);

        assert_eq!(gc.count_objects(), 3);

        gc.collect_garbage(&[a.clone()]);
        assert_eq!(gc.count_objects(), 3);

        a.borrow_mut().children.borrow_mut().retain(|child| {
            if let Some(child_rc) = child.upgrade() {
                !Rc::ptr_eq(&child_rc, &c)
            } else {
                true
            }
        });

        b.borrow_mut().children.borrow_mut().retain(|child| {
            if let Some(child_rc) = child.upgrade() {
                !Rc::ptr_eq(&child_rc, &c)
            } else {
                true
            }
        });

        gc.collect_garbage(&[a.clone()]);
        assert_eq!(gc.count_objects(), 2);
    }

    #[test]
    fn test_gc_removes_isolated_node_immediately() {
        let gc = Gc::new();
        let orphan = Node::new("Orphan");
        gc.register(&orphan);

        assert_eq!(gc.count_objects(), 1);

        gc.collect_garbage(&[]);
        assert_eq!(gc.count_objects(), 0);
    }

    #[test]
    fn test_gc_with_circular_references() {
        let gc = Gc::new();
        let a = Node::new("A");
        let b = Node::new("B");

        gc.register(&a);
        gc.register(&b);

        Node::add_child(&a, &b);
        Node::add_child(&b, &a);

        gc.collect_garbage(&[a.clone()]);
        assert_eq!(gc.count_objects(), 2);

        gc.collect_garbage(&[]);
        assert_eq!(gc.count_objects(), 0);
    }

    #[test]
    fn test_gc_removes_disconnected_child() {
        let gc = Gc::new();

        let a = Node::new("A");
        let b = Node::new("B");
        let c = Node::new("C");

        gc.register(&a);
        gc.register(&b);
        gc.register(&c);

        Node::add_child(&a, &b);
        Node::add_child(&a, &c);
        Node::add_child(&b, &c);

        assert_eq!(gc.count_objects(), 3);

        gc.collect_garbage(&[a.clone()]);
        assert_eq!(gc.count_objects(), 3);

        a.borrow_mut().children.borrow_mut().retain(|child| {
            if let Some(child_rc) = child.upgrade() {
                !Rc::ptr_eq(&child_rc, &c)
            } else {
                true
            }
        });

        b.borrow_mut().children.borrow_mut().retain(|child| {
            if let Some(child_rc) = child.upgrade() {
                !Rc::ptr_eq(&child_rc, &c)
            } else {
                true
            }
        });

        gc.collect_garbage(&[a.clone()]);
        assert_eq!(gc.count_objects(), 2);
    }
}
