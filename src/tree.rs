use crate::*;

#[derive(Clone, Copy)]
pub struct Tree(pub *mut Word);

impl Tree {
  #[inline(always)]
  pub fn root(self) -> UnpackedWord {
    unsafe { *self.0 }.unpack()
  }
  #[inline(always)]
  pub fn offset(self, index: usize) -> Tree {
    unsafe { Tree(self.0.offset(index as isize)) }
  }
  #[inline(always)]
  pub fn node(self, index: usize) -> UnpackedWord {
    self.offset(index).root()
  }
}

delegate_debug!({impl Debug for OwnedTree} (self) => (self.kind(), &*self));

#[derive(Clone, Copy)]
pub struct OwnedTree(pub *mut usize);

impl OwnedTree {
  #[inline(always)]
  pub fn kind(self) -> usize {
    unsafe { *self.0 }
  }
  #[inline(always)]
  pub fn tree(self) -> Tree {
    unsafe { Tree(self.0.offset(1) as *mut Word) }
  }
  #[inline(never)]
  pub fn clone(raw: OwnedTree) -> OwnedTree {
    let kind = raw.kind();
    let tree = raw.tree();
    let len = tree.root().length();
    let mut buffer = Box::<[Word]>::new_uninit_slice(1 + len);
    buffer[0].write(Word(kind));
    unsafe { std::ptr::copy_nonoverlapping(tree.0, &mut buffer[1] as *mut _ as *mut _, len) };
    OwnedTree(Box::into_raw(buffer) as *mut _)
  }
  #[inline(never)]
  pub fn take(kind: usize, tree: Tree) -> OwnedTree {
    let len = tree.root().length();
    let mut buffer = Box::<[Word]>::new_uninit_slice(1 + len);
    buffer[0].write(Word(kind));
    for i in 0..len {
      let word = tree.node(i);
      buffer[i + 1].write(word.pack());
      match word {
        UnpackedWord::Ref(r) => match r.unpack() {
          UnpackedRef::Auxiliary(r) => unsafe {
            *r = UnpackedRef::Auxiliary(&buffer[i + 1] as *const _ as *mut _).pack();
          },
          _ => {}
        },
        _ => {}
      }
    }
    OwnedTree(Box::into_raw(buffer) as *mut _)
  }
  pub fn drop(self) {
    unsafe {
      drop(Box::<[usize]>::from_raw(
        std::ptr::slice_from_raw_parts_mut(self.0, 1 + self.tree().root().length()),
      ));
    }
  }
}
