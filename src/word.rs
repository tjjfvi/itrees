use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Word(pub usize);

delegate_debug!({impl Debug for Word} (self) => self.unpack());

#[derive(Debug, Clone, Copy)]
pub enum UnpackedWord {
  Era,
  Ref(Ref),
  Ctr(usize),
}

impl Word {
  pub const ERA: Word = Word(0);

  #[inline(always)]
  pub fn unpack(self) -> UnpackedWord {
    if self.0 & 1 == 1 {
      UnpackedWord::Ctr(self.0)
    } else if self.0 == 0 {
      UnpackedWord::Era
    } else {
      UnpackedWord::Ref(Ref(self.0))
    }
  }
}

impl UnpackedWord {
  #[inline(always)]
  pub fn pack(self) -> Word {
    match self {
      UnpackedWord::Era => Word::ERA,
      UnpackedWord::Ref(r) => Word(r.0),
      UnpackedWord::Ctr(len) => Word(len | 1),
    }
  }

  #[inline(always)]
  pub fn length(self) -> usize {
    match self {
      UnpackedWord::Ctr(d) => d,
      _ => 1,
    }
  }
}
