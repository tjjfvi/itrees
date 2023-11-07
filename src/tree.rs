use bytemuck::{Pod, Zeroable};
use std::{mem::MaybeUninit, num::NonZeroU16};

use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct TreeRange {
  pub refs_start: u16,
  pub refs_end: u16,
  pub form_start: u16,
  pub form_end: u16,
}

impl std::ops::Add for TreeRange {
  type Output = TreeRange;

  #[inline(always)]
  fn add(self, rhs: Self) -> Self::Output {
    debug_assert!(self.refs_start + rhs.refs_end < self.refs_end);
    debug_assert!(self.form_start + rhs.form_end < self.form_end);
    TreeRange {
      refs_start: self.refs_start + rhs.refs_start,
      refs_end: self.refs_start + rhs.refs_end,
      form_start: self.form_start + rhs.form_start,
      form_end: self.form_start + rhs.form_end,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum UnpackedTreeNode {
  Era,
  Ref(Ref),
  Cup { sign: bool, dist: NonZeroU16 },
  Ctr { left: TreeRange, right: TreeRange },
}

#[derive(Debug, Clone, Copy)]
pub struct TreeSlice<'a> {
  pub refs: &'a [Ref],
  pub form: &'a [Word],
}

impl<'a> TreeSlice<'a> {
  #[inline(always)]
  pub fn slice(&self, range: TreeRange) -> Self {
    TreeSlice {
      refs: &self.refs[range.refs_start as usize..range.refs_end as usize],
      form: &self.form[range.form_start as usize..range.form_end as usize],
    }
  }

  #[inline(always)]
  pub fn unpack_node(self) -> UnpackedTreeNode {
    match self.form[0].unpack() {
      UnpackedWord::Era => UnpackedTreeNode::Era,
      UnpackedWord::Ref => UnpackedTreeNode::Ref(self.refs[0]),
      UnpackedWord::Cup { sign, dist } => UnpackedTreeNode::Cup { sign, dist },
      UnpackedWord::Ctr(_) => {
        let (left_refs_len, left_form_len) = dimensions(&self.form[2..]);
        let left = TreeRange {
          refs_start: 0,
          refs_end: left_refs_len,
          form_start: 2,
          form_end: 2 + left_form_len,
        };
        let right = TreeRange {
          refs_start: left_refs_len,
          refs_end: self.refs.len() as u16,
          form_start: 2 + left_form_len,
          form_end: self.form.len() as u16,
        };
        UnpackedTreeNode::Ctr { left, right }
      }
    }
  }
}

#[inline(always)]
fn dimensions(form: &[Word]) -> (u16, u16) {
  match form[0].unpack() {
    UnpackedWord::Ctr(length) => (form[1].0, 2 + length),
    UnpackedWord::Ref => (1, 1),
    _ => (0, 1),
  }
}

pub struct TreeSliceMut<'a> {
  pub refs: &'a mut [Ref],
  pub form: &'a mut [Word],
}

delegate_debug!({impl<'a> Debug for TreeSliceMut<'a>} (self) => self.borrow());

impl<'a> TreeSliceMut<'a> {
  #[inline(always)]
  pub fn slice(&self, range: TreeRange) -> TreeSlice {
    self.borrow().slice(range)
  }
  #[inline(always)]
  pub fn slice_mut(&mut self, range: TreeRange) -> TreeSliceMut {
    TreeSliceMut {
      refs: &mut self.refs[range.refs_start as usize..range.refs_end as usize],
      form: &mut self.form[range.form_start as usize..range.form_end as usize],
    }
  }
  #[inline(always)]
  pub fn into_slice_mut(self, range: TreeRange) -> Self {
    TreeSliceMut {
      refs: &mut self.refs[range.refs_start as usize..range.refs_end as usize],
      form: &mut self.form[range.form_start as usize..range.form_end as usize],
    }
  }
  #[inline(always)]
  pub fn borrow(&self) -> TreeSlice {
    TreeSlice {
      refs: &*self.refs,
      form: &*self.form,
    }
  }
}

pub type RawTree = *mut u64;

delegate_debug!({impl Debug for Tree} (self) => (self.kind, self.borrow()));

#[derive(Debug, Clone, Copy, Zeroable, Pod, PartialEq, Eq)]
#[repr(C)]
pub struct TreeHeader {
  kind: u32,
  refs_len: u16,
  form_len: u16,
}

impl TreeHeader {
  pub fn length(self) -> usize {
    1 + self.refs_len as usize + (self.form_len as usize + 3 / 4)
  }
}

pub struct Tree {
  pub raw: RawTree,
  pub kind: u32,
  refs: *mut [Ref],
  form: *mut [Word],
}

impl Tree {
  pub fn borrow(&self) -> TreeSlice {
    unsafe {
      TreeSlice {
        refs: &*self.refs,
        form: &*self.form,
      }
    }
  }
  pub fn borrow_mut(&mut self) -> TreeSliceMut {
    unsafe {
      TreeSliceMut {
        refs: &mut *self.refs,
        form: &mut *self.form,
      }
    }
  }
  #[inline(always)]
  pub fn slice(&self, range: TreeRange) -> TreeSlice {
    self.borrow().slice(range)
  }
  #[inline(always)]
  pub fn slice_mut(&mut self, range: TreeRange) -> TreeSliceMut {
    self.borrow_mut().into_slice_mut(range)
  }
}

impl Tree {
  pub unsafe fn from_raw(raw: RawTree) -> Tree {
    let header = *(raw as *const TreeHeader);
    let refs =
      std::ptr::slice_from_raw_parts_mut(raw.offset(1) as *mut Ref, header.refs_len as usize);
    let form = std::ptr::slice_from_raw_parts_mut(
      raw.offset(1 + header.refs_len as isize) as *mut Word,
      header.form_len as usize,
    );
    Tree {
      raw,
      kind: header.kind,
      refs,
      form,
    }
  }
  pub fn from_form(kind: u32, form: &[Word]) -> Tree {
    let (refs_len, form_len) = dimensions(form);
    let header = TreeHeader {
      kind,
      refs_len,
      form_len,
    };
    let mut buffer = Box::<[u64]>::new_uninit_slice(header.length());
    buffer[0].write(bytemuck::must_cast(header));
    buffer[1..1 + refs_len as usize].fill(MaybeUninit::new(0));
    unsafe {
      std::ptr::copy_nonoverlapping(
        form as *const _ as *const Word,
        &mut buffer[1 + refs_len as usize] as *mut _ as *mut _,
        form_len as usize,
      )
    };
    let x = unsafe { Tree::from_raw(Box::into_raw(buffer) as *mut _) };
    assert_eq!(header, x.header());
    x
  }
  pub fn header(&self) -> TreeHeader {
    unsafe {
      TreeHeader {
        kind: self.kind,
        refs_len: (*self.refs).len() as u16,
        form_len: (*self.form).len() as u16,
      }
    }
  }
  pub fn drop(self) {
    unsafe {
      drop(Box::<[u64]>::from_raw(std::ptr::slice_from_raw_parts_mut(
        self.raw,
        self.header().length(),
      )));
    }
  }
}
