#![feature(new_uninit)]

use std::{fmt::Debug, mem::MaybeUninit, num::NonZeroU16, ops::Range, u16};

use bytemuck::{Pod, Zeroable};

macro_rules! unpack_debug {
  ($($impl_line:tt)*) => {
    $($impl_line)* {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.unpack().fmt(f)
      }
    }
  };
}

#[derive(Debug, Clone, Copy)]
enum UnpackedDatum {
  Era,
  Ref,
  Cup { sign: bool, dist: NonZeroU16 },
  Con(u16),
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
struct Datum(u16);

unpack_debug!(impl Debug for Datum);

impl Datum {
  pub const ERA: Datum = Datum(0);
  pub const REF: Datum = Datum(1 << 14);
}

impl Datum {
  #[inline(always)]
  pub fn unpack(self) -> UnpackedDatum {
    match self.0 >> 15 {
      0b0 => {
        let sign = (self.0 >> 14 & 1) != 0;
        let dist = self.0 & 0b00111111_11111111;
        NonZeroU16::new(dist)
          .map(|dist| UnpackedDatum::Cup { sign, dist })
          .unwrap_or(if sign {
            UnpackedDatum::Ref
          } else {
            UnpackedDatum::Era
          })
      }
      0b1 => UnpackedDatum::Con(self.0 & 0b01111111_11111111),
      _ => unreachable!(),
    }
  }
}

impl UnpackedDatum {
  #[inline(always)]
  pub fn pack(self) -> Datum {
    match self {
      UnpackedDatum::Era => Datum::ERA,
      UnpackedDatum::Ref => Datum::REF,
      UnpackedDatum::Cup { sign, dist } => {
        let dist: u16 = dist.into();
        debug_assert!(dist & 0b11000000_00000000 == 0);
        Datum((sign as u16) << 14 | dist)
      }
      UnpackedDatum::Con(x) => {
        debug_assert!(x & 0b10000000_00000000 == 0);
        Datum(x | 0b10000000_00000000)
      }
    }
  }
}

impl Datum {
  #[inline(always)]
  fn dimensions(data: &[Datum]) -> (u16, u16) {
    match data[0].unpack() {
      UnpackedDatum::Con(length) => (2 + length, data[1].0),
      UnpackedDatum::Ref => (1, 1),
      _ => (1, 0),
    }
  }
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
struct Ref(u64);

unpack_debug!(impl Debug for Ref);

impl Ref {
  #[inline(always)]
  pub fn unpack(self) -> UnpackedRef {
    UnpackedRef(
      (self.0 >> 48) as u16,
      (self.0 & 0x_0000ffff_ffffffff) as *mut _,
    )
  }
}

type RawTree = *mut u64;

#[derive(Debug, Clone, Copy)]
struct UnpackedRef(u16, RawTree);

impl UnpackedRef {
  #[inline(always)]
  pub fn pack(self) -> Ref {
    debug_assert!(self.1 as u64 >> 48 == 0);
    Ref(((self.0 as u64) << 48) | self.1 as u64)
  }
}

struct TreeRefMut<'a> {
  kind: u32,
  refs: &'a mut [Ref],
  data: &'a mut [Datum],
}

impl<'a> TreeRefMut<'a> {
  pub fn borrow(&self) -> TreeRef<'a> {
    TreeRef {
      kind: self.kind,
      refs: &*self.refs,
      data: &*self.data,
    }
  }
}

#[derive(Clone, Copy)]
struct TreeRef<'a> {
  kind: u32,
  refs: &'a [Ref],
  data: &'a [Datum],
}

unpack_debug!(impl<'a> Debug for TreeRef<'a>);

#[derive(Debug, Clone, Copy)]
enum UnpackedTreeRef<'a> {
  Era,
  Ref(Ref),
  Cup {
    sign: bool,
    dist: NonZeroU16,
  },
  Con {
    header: Header,
    left: TreeRef<'a>,
    right: TreeRef<'a>,
  },
}

impl<'a> TreeRef<'a> {
  #[inline(always)]
  pub fn unpack(self) -> UnpackedTreeRef<'a> {
    match self.data[0].unpack() {
      UnpackedDatum::Era => UnpackedTreeRef::Era,
      UnpackedDatum::Ref => UnpackedTreeRef::Ref(self.refs[0]),
      UnpackedDatum::Cup { sign, dist } => UnpackedTreeRef::Cup { sign, dist },
      UnpackedDatum::Con(length) => {
        let (left_data_len, left_refs_len) = Datum::dimensions(&self.data[2..]);
        let left = TreeRef {
          kind: self.kind,
          refs: &self.refs[0..left_refs_len as usize],
          data: &self.data[2..2 + left_data_len as usize],
        };
        let right = TreeRef {
          kind: self.kind,
          refs: &self.refs[left_refs_len as usize..],
          data: &self.data[2 + left_data_len as usize..],
        };
        UnpackedTreeRef::Con {
          header: Header {
            kind: self.kind,
            data_len: length,
            refs_len: self.data[1].0,
          },
          left,
          right,
        }
      }
    }
  }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod, PartialEq, Eq)]
#[repr(C)]
struct TreeRange {
  refs_start: u16,
  refs_end: u16,
  data_start: u16,
  data_end: u16,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod, PartialEq, Eq)]
#[repr(C)]
struct Header {
  kind: u32,
  refs_len: u16,
  data_len: u16,
}

impl Header {
  pub fn length(self) -> usize {
    1 + self.refs_len as usize + (self.data_len as usize + 3 / 4)
  }
}

struct OwnedTree {
  raw: RawTree,
  kind: u32,
  refs: *mut [Ref],
  data: *mut [Datum],
}

impl OwnedTree {
  pub unsafe fn from_raw(raw: RawTree) -> OwnedTree {
    let header = *(raw as *const Header);
    let refs =
      std::ptr::slice_from_raw_parts_mut(raw.offset(1) as *mut Ref, header.refs_len as usize);
    let data = std::ptr::slice_from_raw_parts_mut(
      raw.offset(1 + header.refs_len as isize) as *mut Datum,
      header.data_len as usize,
    );
    OwnedTree {
      raw,
      kind: header.kind,
      refs,
      data,
    }
  }
  #[inline(always)]
  pub fn borrow(&self) -> TreeRef {
    TreeRef {
      kind: self.kind,
      refs: self.refs(),
      data: self.data(),
    }
  }
  #[inline(always)]
  pub fn borrow_mut(&mut self) -> TreeRefMut {
    unsafe {
      TreeRefMut {
        kind: self.kind,
        refs: &mut *self.refs,
        data: &mut *self.data,
      }
    }
  }
  #[inline(always)]
  pub fn refs(&self) -> &[Ref] {
    unsafe { &*self.refs }
  }
  #[inline(always)]
  pub fn data(&self) -> &[Datum] {
    unsafe { &*self.data }
  }
  #[inline(always)]
  pub fn refs_mut(&mut self) -> &mut [Ref] {
    unsafe { &mut *self.refs }
  }
  #[inline(always)]
  pub fn data_mut(&mut self) -> &mut [Datum] {
    unsafe { &mut *self.data }
  }
  pub fn from_prototype(kind: u32, data: &[Datum]) -> OwnedTree {
    let (data_len, refs_len) = Datum::dimensions(data);
    let header = Header {
      kind,
      data_len,
      refs_len,
    };
    let mut buffer = Box::<[u64]>::new_uninit_slice(header.length());
    buffer[0].write(bytemuck::must_cast(header));
    buffer[1..1 + refs_len as usize].fill(MaybeUninit::new(0));
    unsafe {
      std::ptr::copy_nonoverlapping(
        data as *const _ as *const Datum,
        &mut buffer[1 + refs_len as usize] as *mut _ as *mut _,
        data_len as usize,
      )
    };
    let x = unsafe { OwnedTree::from_raw(Box::into_raw(buffer) as *mut _) };
    assert_eq!(header, x.header());
    x
  }
  pub fn header(&self) -> Header {
    Header {
      kind: self.kind,
      refs_len: self.refs().len() as u16,
      data_len: self.data().len() as u16,
    }
  }
  pub fn destroy(self) {
    unsafe {
      drop(Box::<[u64]>::from_raw(std::ptr::slice_from_raw_parts_mut(
        self.raw,
        self.header().length(),
      )));
    }
  }
}

struct Net;

impl Net {
  fn link(&mut self, a: Ref, b: Ref) {
    dbg!(a, b);
    // todo!()
  }

  fn bind(&mut self, a: Ref, b: OwnedTree) {
    dbg!(a, b.borrow());
    // todo!()
  }

  fn erase(&mut self, a: Ref) {
    dbg!(a);
    // todo!()
  }

  fn commute(&mut self, mut a: OwnedTree, mut b: OwnedTree) {
    for r in b.refs_mut() {
      let x = OwnedTree::from_prototype(a.kind, a.data());
      self.bind(std::mem::replace(r, Ref(x.raw as u64)), x);
    }
    for r in a.refs_mut() {
      let x = OwnedTree::from_prototype(b.kind, b.data());
      self.bind(std::mem::replace(r, Ref(x.raw as u64)), x);
    }
    for i in 0..a.refs().len() {
      for j in 0..b.refs().len() {
        self.link(
          UnpackedRef(j as u16 + 1, a.refs()[i].0 as RawTree).pack(),
          UnpackedRef(i as u16 + 1, b.refs()[j].0 as RawTree).pack(),
        );
      }
    }
    a.destroy();
    b.destroy();
  }

  fn annihilate(&mut self, mut a: OwnedTree, mut b: OwnedTree) {
    todo!()
    // let mut n = 1;
    // let a = a.borrow_mut();
    // let b = b.borrow_mut();
    // match (a.borrow().unpack(), b.borrow().unpack()) {
    //   (UnpackedTreeRef::Era, UnpackedTreeRef::Era) => {}
    //   (UnpackedTreeRef::Era, Unpack => {
    //     for &x in b.refs() {
    //       self.erase(x);
    //     }
    //     b.destroy();
    //   }
    //   UnpackedTreeRef::Con {
    //     header,
    //     left,
    //     right,
    //   } => {}
    //   _ => todo!(),
    // }
  }
}

fn main() {
  let data = &[
    UnpackedDatum::Con(11).pack(),
    Datum(4),
    UnpackedDatum::Con(5).pack(),
    Datum(3),
    Datum::REF,
    UnpackedDatum::Con(2).pack(),
    Datum(2),
    Datum::REF,
    Datum::REF,
    UnpackedDatum::Con(2).pack(),
    Datum(1),
    Datum::ERA,
    Datum::REF,
  ];
  let a = OwnedTree::from_prototype(0, data);
  let b = OwnedTree::from_prototype(1, data);
  Net.commute(a, b);
  // println!(
  //   "{:#?}",
  //   TreeRef {
  //     kind: 0,
  //     data,
  //     refs: &[
  //       UnpackedRef(0, std::ptr::null_mut()).pack(),
  //       UnpackedRef(1, std::ptr::null_mut()).pack(),
  //       UnpackedRef(2, std::ptr::null_mut()).pack(),
  //       UnpackedRef(3, std::ptr::null_mut()).pack(),
  //     ]
  //   }
  // );
}
