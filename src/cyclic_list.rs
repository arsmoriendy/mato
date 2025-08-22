use std::{cell::RefCell, rc::Rc};

type NodeRef<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    pub data: T,
    prev: NodeRef<T>,
    next: NodeRef<T>,
}

impl<T> Node<T> {
    fn new<'a>(data: T) -> Node<T> {
        Node {
            data,
            prev: None,
            next: None,
        }
    }
}

pub struct CyclicList<T> {
    head: NodeRef<T>,
    tail: NodeRef<T>,
    current: NodeRef<T>,
}

impl<T> Default for CyclicList<T> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            current: None,
        }
    }
}

impl<T> CyclicList<T> {
    pub fn push_back(&mut self, data: T) {
        let mut new_node = Node::new(data);

        if self.head.is_none() || self.tail.is_none() {
            self.head = Some(Rc::new(RefCell::new(new_node)));
            self.tail = self.head.clone();
            self.current = self.head.clone();
            return;
        }

        new_node.prev = self.tail.clone();
        new_node.next = self.head.clone();

        let new_node_ref = Some(Rc::new(RefCell::new(new_node)));

        // TODO: ref instead of clone
        self.tail.clone().unwrap().borrow_mut().next = new_node_ref.clone();
        self.tail = new_node_ref;
    }

    pub fn current<'a>(&self) -> NodeRef<T> {
        Some(self.current.clone()?)
    }

    pub fn advance(&mut self) {
        let current = self.current().unwrap();
        self.current = current.borrow().next.clone();
    }
}
