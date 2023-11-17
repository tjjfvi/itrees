use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct PackedRef(pub usize);

delegate_debug!({impl Debug for PackedRef} (self) => self.unpack());

impl PackedRef {
  pub const NULL: PackedRef = PackedRef(0);
  #[inline(always)]
  pub fn unpack(self) -> Ref {
    if self.0 & 0b10 != 0 {
      Ref::Principal(OwnedTree((self.0 & !0b10) as _))
    } else {
      Ref::Auxiliary(self.0 as _)
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Ref {
  Principal(OwnedTree),
  Auxiliary(*mut PackedRef),
}

impl Ref {
  #[inline(always)]
  pub fn pack(self) -> PackedRef {
    match self {
      Ref::Principal(p) => {
        let p = p.0 as usize;
        debug_assert!(p & 0b10 == 0);
        PackedRef(p | 0b10)
      }
      Ref::Auxiliary(p) => {
        let p = p as usize;
        debug_assert!(p & 0b10 == 0);
        PackedRef(p)
      }
    }
  }
}
