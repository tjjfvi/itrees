use crate::*;
use bytemuck::{Pod, Zeroable};
use std::num::NonZeroU16;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Word(pub u16);

delegate_debug!({impl Debug for Word} (self) => self.unpack());

#[derive(Debug, Clone, Copy)]
pub enum UnpackedWord {
  Era,
  Ref,
  Cup { sign: bool, dist: NonZeroU16 },
  Ctr(u16),
}

impl Word {
  pub const ERA: Word = Word(0);
  pub const REF: Word = Word(1 << 14);

  #[inline(always)]
  pub fn unpack(self) -> UnpackedWord {
    match self.0 >> 15 {
      0b0 => {
        let sign = (self.0 >> 14 & 1) != 0;
        let dist = self.0 & 0b00111111_11111111;
        NonZeroU16::new(dist)
          .map(|dist| UnpackedWord::Cup { sign, dist })
          .unwrap_or(if sign {
            UnpackedWord::Ref
          } else {
            UnpackedWord::Era
          })
      }
      0b1 => UnpackedWord::Ctr(self.0 & 0b01111111_11111111),
      _ => unreachable!(),
    }
  }
}

impl UnpackedWord {
  #[inline(always)]
  pub fn pack(self) -> Word {
    match self {
      UnpackedWord::Era => Word::ERA,
      UnpackedWord::Ref => Word::REF,
      UnpackedWord::Cup { sign, dist } => {
        let dist: u16 = dist.into();
        debug_assert!(dist & 0b11000000_00000000 == 0);
        Word((sign as u16) << 14 | dist)
      }
      UnpackedWord::Ctr(x) => {
        debug_assert!(x & 0b10000000_00000000 == 0);
        Word(x | 0b10000000_00000000)
      }
    }
  }
}
