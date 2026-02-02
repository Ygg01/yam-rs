use alloc::rc::Weak;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use yam_common::YamlDoc;

pub type WeakHandle<'input> = Weak<Node<'input>>;

#[derive(Default)]
pub struct Node<'input> {
    data: YamlDoc<'input>,
    parent: Cell<Option<WeakHandle<'input>>>,
    children: RefCell<Vec<WeakHandle<'input>>>,
}
