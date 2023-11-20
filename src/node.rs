use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct PackedNode(pub usize);

delegate_debug!({impl Debug for PackedNode} (self) => self.unpack());

#[derive(Debug, Clone, Copy)]
pub enum Node {
  Era,
  Principal(Tree),
  Auxiliary(Tree),
  Ctr(usize),
}

impl PackedNode {
  pub const ERA: PackedNode = PackedNode(0);

  #[inline(always)]
  pub fn unpack(self) -> Node {
    if self.0 & 1 == 1 {
      Node::Ctr(self.0 >> 1)
    } else if self.0 == 0 {
      Node::Era
    } else if self.0 & 0b10 != 0 {
      Node::Principal(Tree((self.0 & !0b10) as _))
    } else {
      Node::Auxiliary(Tree(self.0 as _))
    }
  }
}

impl Node {
  #[inline(always)]
  pub fn pack(self) -> PackedNode {
    match self {
      Node::Era => PackedNode::ERA,
      Node::Principal(r) => PackedNode(r.0 as usize | 0b10),
      Node::Auxiliary(r) => PackedNode(r.0 as usize),
      Node::Ctr(kind) => PackedNode(kind << 1 | 1),
    }
  }
}
