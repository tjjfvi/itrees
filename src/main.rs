#![feature(new_uninit)]

mod parse;
mod print;
mod r#ref;
mod tree;
mod utils;
mod word;

use logos::Logos;
use parse::*;
use print::*;
use r#ref::*;
use tree::*;
use word::*;

use std::{
  fmt::Debug,
  time::{Duration, Instant},
};

#[derive(Default, Debug)]
struct Net {
  active: Vec<(RawFullTree, RawFullTree)>,
  av: Vec<(usize, OwnedTree)>,
  bv: Vec<(usize, OwnedTree)>,
}

impl Net {
  fn link(&mut self, a: Ref, b: Ref) {
    match (a.unpack(), b.unpack()) {
      (UnpackedRef::Principal(a), UnpackedRef::Principal(b)) => self.active.push((a, b)),
      (UnpackedRef::Principal(_), UnpackedRef::Auxiliary(b)) => unsafe { *b = a },
      (UnpackedRef::Auxiliary(a), UnpackedRef::Principal(_)) => unsafe { *a = b },
      (UnpackedRef::Auxiliary(aa), UnpackedRef::Auxiliary(ba)) => unsafe {
        *aa = b;
        *ba = a;
      },
    }
  }

  fn bind(&mut self, a: Ref, b: OwnedTree) {
    match a.unpack() {
      UnpackedRef::Principal(a) => self.active.push((a, b.raw)),
      UnpackedRef::Auxiliary(a) => unsafe { *a = UnpackedRef::Principal(b.raw).pack() },
    }
  }

  fn erase(&mut self, a: Ref) {
    match a.unpack() {
      UnpackedRef::Auxiliary(a) => unsafe { *a = Ref::NULL },
      UnpackedRef::Principal(a) => self.active.push((
        a,
        OwnedTree::clone(&mut [unsafe { *a }, Word::ERA.0] as *mut _).raw,
      )),
    }
  }

  pub fn reduce_one(&mut self) -> Option<()> {
    let (a, b) = self.active.pop()?;
    let (a, b) = (OwnedTree::from_raw(a), OwnedTree::from_raw(b));
    if a.kind == b.kind {
      self.annihilate(a, b);
    } else {
      self.commute(a, b);
    }
    Some(())
  }

  #[inline(never)]
  fn commute(&mut self, a: OwnedTree, b: OwnedTree) {
    let mut av = std::mem::take(&mut self.av);
    let mut bv = std::mem::take(&mut self.bv);
    av.extend(
      (0..unsafe { *get_tree(a.raw) }.unpack().length())
        .map(|i| (i, unsafe { *get_tree(a.raw).offset(i as isize) }))
        .filter(|(_, x)| matches!(x.unpack(), UnpackedWord::Ref(_)))
        .map(|(i, _)| (i, OwnedTree::clone(b.raw))),
    );
    bv.extend(
      (0..unsafe { *get_tree(b.raw) }.unpack().length())
        .map(|i| (i, unsafe { *get_tree(b.raw).offset(i as isize) }))
        .filter(|(_, x)| matches!(x.unpack(), UnpackedWord::Ref(_)))
        .map(|(i, _)| (i, OwnedTree::clone(a.raw))),
    );
    for &(ai, ref bc) in av.iter() {
      for &(bj, ref ac) in bv.iter() {
        self.link(
          UnpackedRef::Auxiliary(unsafe { get_tree(ac.raw).offset(ai as isize) as *mut _ }).pack(),
          UnpackedRef::Auxiliary(unsafe { get_tree(bc.raw).offset(bj as isize) as *mut _ }).pack(),
        )
      }
    }
    for (ai, b) in av.drain(..) {
      self.bind(Ref(unsafe { *get_tree(a.raw).offset(ai as isize) }.0), b)
    }
    for (bi, a) in bv.drain(..) {
      self.bind(Ref(unsafe { *get_tree(b.raw).offset(bi as isize) }.0), a)
    }
    a.drop();
    b.drop();
    self.av = av;
    self.bv = bv;
  }

  #[inline(never)]
  fn annihilate(&mut self, a: OwnedTree, b: OwnedTree) {
    let kind = a.kind;
    {
      let mut a = get_tree(a.raw);
      let mut b = get_tree(b.raw);
      let mut n = 1usize;
      let mut a_era_stack = 0usize;
      let mut b_era_stack = 0usize;
      while n > 0 {
        match unsafe { ((*a).unpack(), (*b).unpack()) } {
          (UnpackedWord::Era, UnpackedWord::Era) => {}
          (UnpackedWord::Era, UnpackedWord::Ref(r)) => self.erase(r),
          (UnpackedWord::Ref(r), UnpackedWord::Era) => self.erase(r),
          (UnpackedWord::Ref(a), UnpackedWord::Ref(b)) => self.link(a, b),
          (UnpackedWord::Era, UnpackedWord::Ctr(_)) => {
            n += 2;
            a_era_stack += 2;
          }
          (UnpackedWord::Ctr(_), UnpackedWord::Era) => {
            n += 2;
            b_era_stack += 2
          }
          (UnpackedWord::Ctr(_), UnpackedWord::Ctr(_)) => n += 2,
          (UnpackedWord::Ref(r), UnpackedWord::Ctr(l)) => {
            self.bind(r, OwnedTree::take(kind, b));
            b = unsafe { b.offset((l - 1) as isize) };
          }
          (UnpackedWord::Ctr(l), UnpackedWord::Ref(r)) => {
            self.bind(r, OwnedTree::take(kind, a));
            a = unsafe { a.offset((l - 1) as isize) };
          }
        }
        if a_era_stack != 0 {
          a_era_stack -= 1
        } else {
          a = unsafe { a.offset(1) }
        }
        if b_era_stack != 0 {
          b_era_stack -= 1
        } else {
          b = unsafe { b.offset(1) }
        }
        n -= 1;
      }
    }
    a.drop();
    b.drop();
  }
}

fn main() {
  let program = include_str!("../programs/dec_bits_comp.ic");

  let mut d = Duration::ZERO;
  let n = 100;

  let mut a = &mut [] as *mut _;
  let mut b = Net::default();
  for _ in 0..n {
    (a, b) = parse_program(&mut Token::lexer(program)).unwrap();

    // println!("{:?}", PrintNet(&*a, &b));

    b.av.reserve(100);
    b.bv.reserve(100);

    let start = Instant::now();

    let n = inner(&mut b);

    d += start.elapsed();

    println!("{} steps ({:?})\n", n, start.elapsed());
  }

  unsafe {
    println!("{:?}", PrintNet(&*a, &b));
  }

  dbg!(d / n);
}

#[inline(never)]
fn inner(b: &mut Net) -> i32 {
  let mut n = 0;
  while let Some(_) = b.reduce_one() {
    n += 1;
  }
  n
}
