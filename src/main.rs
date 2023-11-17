#![feature(new_uninit)]

mod node;
mod parse;
mod print;
mod r#ref;
mod tree;
mod utils;

use logos::Logos;
use node::*;
use parse::*;
use print::*;
use r#ref::*;
use tree::*;

use std::{
  fmt::Debug,
  time::{Duration, Instant},
};

#[derive(Default, Debug)]
struct Net {
  active: Vec<(OwnedTree, OwnedTree)>,
  av: Vec<(usize, OwnedTree)>,
  bv: Vec<(usize, OwnedTree)>,
}

impl Net {
  #[inline(always)]
  fn link(&mut self, a: Ref, b: Ref) {
    match (a, b) {
      (Ref::Principal(a), Ref::Principal(b)) => self.active.push((a, b)),
      (Ref::Principal(_), Ref::Auxiliary(b)) => unsafe { *b = a.pack() },
      (Ref::Auxiliary(a), Ref::Principal(_)) => unsafe { *a = b.pack() },
      (Ref::Auxiliary(aa), Ref::Auxiliary(ba)) => unsafe {
        *aa = b.pack();
        *ba = a.pack();
      },
    }
  }

  #[inline(always)]
  fn bind(&mut self, a: Ref, b: OwnedTree) {
    match a {
      Ref::Principal(a) => self.active.push((a, b)),
      Ref::Auxiliary(a) => unsafe { *a = Ref::Principal(b).pack() },
    }
  }

  #[inline(always)]
  fn erase(&mut self, a: Ref) {
    match a {
      Ref::Auxiliary(a) => unsafe { *a = PackedRef::NULL },
      Ref::Principal(a) => self.active.push((
        a,
        OwnedTree::clone(OwnedTree(&mut [a.kind(), Node::Era.pack().0] as *mut _)),
      )),
    }
  }

  pub fn reduce_one(&mut self) -> Option<()> {
    let (a, b) = self.active.pop()?;
    if a.kind() == b.kind() {
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
      (0..a.tree().root().length())
        .map(|i| (i, a.tree().node(i)))
        .filter(|(_, x)| matches!(x, Node::Ref(_)))
        .map(|(i, _)| (i, OwnedTree::clone(b))),
    );
    bv.extend(
      (0..b.tree().root().length())
        .map(|i| (i, b.tree().node(i)))
        .filter(|(_, x)| matches!(x, Node::Ref(_)))
        .map(|(i, _)| (i, OwnedTree::clone(a))),
    );
    for &(ai, ref bc) in av.iter() {
      for &(bj, ref ac) in bv.iter() {
        self.link(
          Ref::Auxiliary(ac.tree().offset(ai).0 as *mut _),
          Ref::Auxiliary(bc.tree().offset(bj).0 as *mut _),
        )
      }
    }
    for (ai, b) in av.drain(..) {
      self.bind(PackedRef(a.tree().node(ai).pack().0).unpack(), b)
    }
    for (bi, a) in bv.drain(..) {
      self.bind(PackedRef(b.tree().node(bi).pack().0).unpack(), a)
    }
    a.drop();
    b.drop();
    self.av = av;
    self.bv = bv;
  }

  #[inline(never)]
  fn annihilate(&mut self, a: OwnedTree, b: OwnedTree) {
    let kind = a.kind();
    {
      let mut a = a.tree();
      let mut b = b.tree();
      let mut n = 1usize;
      let mut a_era_stack = 0usize;
      let mut b_era_stack = 0usize;
      while n > 0 {
        match (a.root(), b.root()) {
          (Node::Era, Node::Era) => {}
          (Node::Era, Node::Ref(r)) => self.erase(r),
          (Node::Ref(r), Node::Era) => self.erase(r),
          (Node::Ref(a), Node::Ref(b)) => self.link(a, b),
          (Node::Era, Node::Ctr(_)) => {
            n += 2;
            a_era_stack += 2;
          }
          (Node::Ctr(_), Node::Era) => {
            n += 2;
            b_era_stack += 2
          }
          (Node::Ctr(_), Node::Ctr(_)) => n += 2,
          (Node::Ref(r), Node::Ctr(l)) => {
            self.bind(r, OwnedTree::take(kind, b));
            b = b.offset(l - 1);
          }
          (Node::Ctr(l), Node::Ref(r)) => {
            self.bind(r, OwnedTree::take(kind, a));
            a = a.offset(l - 1);
          }
        }
        if a_era_stack != 0 {
          a_era_stack -= 1
        } else {
          a = a.offset(1)
        }
        if b_era_stack != 0 {
          b_era_stack -= 1
        } else {
          b = b.offset(1)
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

    // unsafe {
    //   println!("{:?}", PrintNet(&*a, &b));
    // }

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
