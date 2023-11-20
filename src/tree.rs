use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Tree(pub *mut PackedNode);

impl Tree {
  pub const ERA: Tree = Tree(&PackedNode::ERA as *const _ as *mut _);
  pub const NULL: Tree = Tree(std::ptr::null_mut());

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
  pub fn kind(self) -> Option<usize> {
    match self.root() {
      Node::Ctr(kind) => Some(kind),
      _ => None,
    }
  }
  #[inline(never)]
  pub fn clone(tree: Tree, len: usize) -> Tree {
    let mut buffer = Box::<[usize]>::new_uninit_slice(len);
    unsafe { std::ptr::copy_nonoverlapping(tree.0, &mut buffer[0] as *mut _ as *mut _, len) };
    Tree(Box::into_raw(buffer) as *mut _)
  }
}
