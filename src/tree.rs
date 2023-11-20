use std::hint::unreachable_unchecked;

use crate::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Tree(pub(crate) *mut PackedNode);

impl Tree {
  pub(crate) const NULL: Tree = Tree(std::ptr::null_mut());

  #[inline(always)]
  pub(crate) fn node(self) -> Node {
    unsafe { *self.0 }.unpack()
  }
  #[inline(always)]
  pub(crate) fn offset(self, index: usize) -> Tree {
    unsafe { Tree(self.0.offset(index as isize)) }
  }
  #[inline(always)]
  pub(crate) fn kind(self) -> usize {
    match self.node() {
      Node::Ctr(kind) => kind,
      _ => unsafe { unreachable_unchecked() },
    }
  }
  #[inline(never)]
  pub(crate) fn new(data: &[PackedNode]) -> Tree {
    Tree(Box::into_raw(Box::<[_]>::from(data)) as *mut _)
  }
}
