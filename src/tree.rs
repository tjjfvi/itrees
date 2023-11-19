use crate::*;
use std::mem::size_of;

#[derive(Debug, Clone, Copy)]
pub struct Tree(pub *mut PackedNode);

impl Tree {
  pub const ERA: Tree = Tree(&PackedNode::ERA as *const _ as *mut _);
  #[inline(always)]
  pub fn root(self) -> Node {
    unsafe { *self.0 }.unpack()
  }
  #[inline(always)]
  pub fn offset(self, index: usize) -> Tree {
    unsafe { Tree(self.0.offset(index as isize)) }
  }
  #[inline(always)]
  pub fn node(self, index: usize) -> Node {
    self.offset(index).root()
  }
  #[inline(always)]
  pub fn contains(self, tree: Tree) -> bool {
    (self.0 as usize..self.0 as usize + self.root().length() * size_of::<usize>())
      .contains(&(tree.0 as usize))
  }
  #[inline(always)]
  pub fn kind(self) -> Option<usize> {
    match self.root() {
      Node::Ctr(_, kind) => Some(kind),
      _ => None,
    }
  }
  #[inline(never)]
  pub fn clone(raw: Tree) -> Tree {
    let tree = raw;
    let len = tree.root().length();
    let mut buffer = Box::<[usize]>::new_uninit_slice(len);
    unsafe { std::ptr::copy_nonoverlapping(tree.0, &mut buffer[0] as *mut _ as *mut _, len) };
    Tree(Box::into_raw(buffer) as *mut _)
  }
}
