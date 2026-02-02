use alloc::rc::Weak;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use yam_common::YamlDoc;

pub type WeakHandle<'input> = Weak<Node<'input>>;

#[derive(Default)]
pub struct Node<'input> {
    _data: YamlDoc<'input>,
    _parent: Cell<Option<WeakHandle<'input>>>,
    _children: RefCell<Vec<WeakHandle<'input>>>,
}
