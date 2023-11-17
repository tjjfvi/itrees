use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct PackedNode(pub usize);

delegate_debug!({impl Debug for PackedNode} (self) => self.unpack());

#[derive(Debug, Clone, Copy)]
pub enum Node {
  Era,
  Ref(Ref),
  Ctr(usize),
}

impl PackedNode {
  pub const ERA: PackedNode = PackedNode(0);

  #[inline(always)]
  pub fn unpack(self) -> Node {
    if self.0 & 1 == 1 {
      Node::Ctr(self.0)
    } else if self.0 == 0 {
      Node::Era
    } else {
      Node::Ref(PackedRef(self.0).unpack())
    }
  }
}

impl Node {
  #[inline(always)]
  pub fn pack(self) -> PackedNode {
    match self {
      Node::Era => PackedNode::ERA,
      Node::Ref(r) => PackedNode(r.pack().0),
      Node::Ctr(len) => PackedNode(len | 1),
    }
  }

  #[inline(always)]
  pub fn length(self) -> usize {
    match self {
      Node::Ctr(d) => d,
      _ => 1,
    }
  }
}
