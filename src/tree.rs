use bytemuck::{Pod, Zeroable};
use std::ops::{Deref, DerefMut};

use crate::*;

pub type Tree = [Word];

#[derive(Debug, Clone, Copy)]
pub enum UnpackedTreeNode {
  Era,
  Ref(Ref),
  Ctr(usize, usize),
}

pub fn unpack_node(tree: &Tree) -> UnpackedTreeNode {
  match tree[0].unpack() {
    UnpackedWord::Era => UnpackedTreeNode::Era,
    UnpackedWord::Ref(r) => UnpackedTreeNode::Ref(r),
    UnpackedWord::Ctr(_) => {
      let left_len = tree[1].unpack().length();
      UnpackedTreeNode::Ctr(1, 1 + left_len)
    }
  }
}

pub type RawTree = *mut usize;

delegate_debug!({impl Debug for OwnedTree} (self) => (self.kind, &*self));

pub struct OwnedTree {
  pub raw: RawTree,
  pub kind: usize,
  tree: *mut [Word],
}

impl Deref for OwnedTree {
  type Target = Tree;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.tree }
  }
}

impl DerefMut for OwnedTree {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.tree }
  }
}

impl OwnedTree {
  pub unsafe fn from_raw(raw: RawTree) -> OwnedTree {
    let kind = *raw;
    let tree = raw.offset(1) as *mut Word;
    let tree = std::ptr::slice_from_raw_parts_mut(tree, (*tree).unpack().length());
    OwnedTree { raw, kind, tree }
  }
  #[inline(never)]
  pub fn new(kind: usize, tree: &Tree) -> OwnedTree {
    let len = tree[0].unpack().length();
    let mut buffer = Box::<[usize]>::new_uninit_slice(1 + len);
    buffer[0].write(kind);
    unsafe {
      std::ptr::copy_nonoverlapping(
        tree as *const _ as *const Word,
        &mut buffer[1] as *mut _ as *mut _,
        len,
      )
    };
    unsafe { OwnedTree::from_raw(Box::into_raw(buffer) as *mut _) }
  }
  #[inline(never)]
  pub fn take(kind: usize, tree: &Tree) -> OwnedTree {
    let len = tree[0].unpack().length();
    let mut buffer = Box::<[usize]>::new_uninit_slice(1 + len);
    buffer[0].write(kind);
    for i in 0..len {
      buffer[i + 1].write(tree[i].0);
      match tree[i].unpack() {
        UnpackedWord::Ref(r) => match r.unpack() {
          UnpackedRef::Auxiliary(r) => unsafe {
            *r = UnpackedRef::Auxiliary(&buffer[i + 1] as *const _ as *mut _).pack();
          },
          _ => {}
        },
        _ => {}
      }
    }
    unsafe { OwnedTree::from_raw(Box::into_raw(buffer) as *mut _) }
  }
  pub fn drop(self) {
    unsafe {
      drop(Box::<[usize]>::from_raw(
        std::ptr::slice_from_raw_parts_mut(self.raw, 1 + (&*self.tree).len()),
      ));
    }
  }
}
