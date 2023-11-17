use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Ref(pub usize);

delegate_debug!({impl Debug for Ref} (self) => self.unpack());

impl Ref {
  pub const NULL: Ref = Ref(0);
  #[inline(always)]
  pub fn unpack(self) -> UnpackedRef {
    if self.0 & 0b10 != 0 {
      UnpackedRef::Principal(OwnedTree((self.0 & !0b10) as _))
    } else {
      UnpackedRef::Auxiliary(self.0 as _)
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum UnpackedRef {
  Principal(OwnedTree),
  Auxiliary(*mut Ref),
}

impl UnpackedRef {
  #[inline(always)]
  pub fn pack(self) -> Ref {
    match self {
      UnpackedRef::Principal(p) => {
        let p = p.0 as usize;
        debug_assert!(p & 0b10 == 0);
        Ref(p | 0b10)
      }
      UnpackedRef::Auxiliary(p) => {
        let p = p as usize;
        debug_assert!(p & 0b10 == 0);
        Ref(p)
      }
    }
  }
}
