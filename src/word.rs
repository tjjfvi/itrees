use crate::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Word(pub u16);

delegate_debug!({impl Debug for Word} (self) => self.unpack());

#[derive(Debug, Clone, Copy)]
pub struct Cup {
  delve: u8,
  path: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
  pub refs_len: u8,
  pub form_len: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum UnpackedWord {
  Era,
  Ref,
  Cup(Cup),
  Ctr(Dimensions),
}

impl Word {
  pub const ERA: Word = Word(0);
  pub const REF: Word = Word(1 << 14);

  #[inline(always)]
  pub fn unpack(self) -> UnpackedWord {
    let mode = (self.0 >> 15) == 1;
    let upper = (self.0 >> 10 & 0b11111) as u8;
    let lower = self.0 & 0b1111111111;
    if mode {
      UnpackedWord::Ctr(Dimensions {
        refs_len: upper,
        form_len: lower,
      })
    } else if lower == 0 {
      if upper == 0 {
        UnpackedWord::Era
      } else {
        UnpackedWord::Ref
      }
    } else {
      UnpackedWord::Cup(Cup {
        delve: upper,
        path: lower,
      })
    }
  }
}

impl UnpackedWord {
  #[inline(always)]
  pub fn pack(self) -> Word {
    let (mode, upper, lower) = match self {
      UnpackedWord::Era => (false, 0, 0),
      UnpackedWord::Ref => (false, 1, 0),
      UnpackedWord::Cup(Cup { delve, path }) => {
        debug_assert!(path != 0);
        (false, delve, path)
      }
      UnpackedWord::Ctr(Dimensions { refs_len, form_len }) => (true, refs_len, form_len),
    };
    debug_assert!(upper == upper & 0b11111);
    debug_assert!(lower == lower & 0b1111111111);
    Word(((mode as u16) << 15) | ((upper as u16) << 10) | lower)
  }

  #[inline(always)]
  pub fn dimensions(self) -> Dimensions {
    match self {
      UnpackedWord::Ctr(d) => d,
      UnpackedWord::Ref => Dimensions {
        refs_len: 1,
        form_len: 1,
      },
      _ => Dimensions {
        refs_len: 0,
        form_len: 1,
      },
    }
  }
}
