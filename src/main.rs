use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    name: String,
    children: RefCell<Vec<Weak<RefCell<Node>>>>,
    marked: RefCell<bool>,
}

impl Node {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            name: name.to_string(),
            children: RefCell::new(Vec::new()),
            marked: RefCell::new(false),
        }))
    }

    fn add_child(parent: &Rc<RefCell<Node>>, child: &Rc<RefCell<Node>>) {
        parent.borrow_mut().children.borrow_mut().push(Rc::downgrade(child));
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

    fn mark(&self, root: &Rc<RefCell<Node>>) {
        let mut stack = vec![Rc::clone(root)];
        while let Some(node) = stack.pop() {
            let node_ref = node.borrow();
            if *node_ref.marked.borrow() {
                continue;
            }
            *node_ref.marked.borrow_mut() = true;
            for weak_child in node_ref.children.borrow().iter() {
                if let Some(child) = weak_child.upgrade() {
                    stack.push(child);
                }
            }
        }
    }

    fn sweep(&self) {
        let mut objects = self.objects.borrow_mut();
        objects.retain(|weak_node| {
            if let Some(node) = weak_node.upgrade() {
                let is_marked = *node.borrow().marked.borrow();
                if is_marked {
                    *node.borrow().marked.borrow_mut() = false;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });
    }

    fn collect_garbage(&self, roots: &[Rc<RefCell<Node>>]) {
        for root in roots {
            self.mark(root);
        }
        self.sweep();
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

    a.borrow_mut().children.borrow_mut().retain(|child| child.upgrade() != Some(c.clone()));

    gc.collect_garbage(&[a.clone()]);
    println!("C削除後: {} 個のオブジェクトが登録されている", gc.count_objects());
}