use crate::*;

pub type Tree = *mut Word;

pub type RawFullTree = *mut usize;

delegate_debug!({impl Debug for OwnedTree} (self) => (self.kind, &*self));

pub struct OwnedTree {
  pub raw: RawFullTree,
  pub kind: usize,
  tree: *mut [Word],
}

pub fn get_tree(raw: RawFullTree) -> Tree {
  unsafe { raw.offset(1) as *mut Word }
}

// impl Deref for OwnedTree {
//   type Target = Tree;
//   fn deref(&self) -> &Self::Target {
//     unsafe { &*self.tree }
//   }
// }

// impl DerefMut for OwnedTree {
//   fn deref_mut(&mut self) -> &mut Self::Target {
//     unsafe { &mut *self.tree }
//   }
// }

impl OwnedTree {
  pub fn from_raw(raw: RawFullTree) -> OwnedTree {
    unsafe {
      let kind = *raw;
      let tree = get_tree(raw);
      let tree = std::ptr::slice_from_raw_parts_mut(tree, (*tree).unpack().length());
      OwnedTree { raw, kind, tree }
    }
  }
  #[inline(never)]
  pub fn clone(raw: RawFullTree) -> OwnedTree {
    let kind = unsafe { *raw };
    let tree = get_tree(raw);
    let len = unsafe { (*tree).unpack().length() };
    let mut buffer = Box::<[Word]>::new_uninit_slice(1 + len);
    buffer[0].write(Word(kind));
    unsafe { std::ptr::copy_nonoverlapping(tree, &mut buffer[1] as *mut _ as *mut _, len) };
    OwnedTree::from_raw(Box::into_raw(buffer) as *mut _)
  }
  #[inline(never)]
  pub fn take(kind: usize, tree: *mut Word) -> OwnedTree {
    let len = unsafe { (*tree).unpack().length() };
    let mut buffer = Box::<[Word]>::new_uninit_slice(1 + len);
    buffer[0].write(Word(kind));
    for i in 0..len {
      let word = unsafe { *tree.offset(i as isize) };
      buffer[i + 1].write(word);
      match word.unpack() {
        UnpackedWord::Ref(r) => match r.unpack() {
          UnpackedRef::Auxiliary(r) => unsafe {
            *r = UnpackedRef::Auxiliary(&buffer[i + 1] as *const _ as *mut _).pack();
          },
          _ => {}
        },
        _ => {}
      }
    }
    OwnedTree::from_raw(Box::into_raw(buffer) as *mut _)
  }
  pub fn drop(self) {
    unsafe {
      drop(Box::<[usize]>::from_raw(
        std::ptr::slice_from_raw_parts_mut(self.raw, 1 + (&*self.tree).len()),
      ));
    }
  }
}
