use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Ref(pub u64);

delegate_debug!({impl Debug for Ref} (self) => self.unpack());

impl Ref {
  #[inline(always)]
  pub fn unpack(self) -> UnpackedRef {
    UnpackedRef(
      (self.0 >> 48) as u16,
      (self.0 & 0x_0000ffff_ffffffff) as *mut _,
    )
  }
}

#[derive(Debug, Clone, Copy)]
pub struct UnpackedRef(pub u16, pub RawTree);

impl UnpackedRef {
  #[inline(always)]
  pub fn pack(self) -> Ref {
    debug_assert!(self.1 as u64 >> 48 == 0);
    Ref(((self.0 as u64) << 48) | self.1 as u64)
  }
}
