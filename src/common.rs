use std::{cell::RefCell, rc::Rc};

pub(crate) type Link<K> = Option<NodeRef<K>>;

pub(crate) type NodeRef<K> = Rc<RefCell<Node<K>>>;

#[derive(Debug)]
pub(crate) struct Node<K> {
    pub(crate) key: K,
    pub(crate) next: Link<K>,
    pub(crate) prev: Link<K>,
}
