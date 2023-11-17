#![feature(new_uninit)]

mod node;
mod parse;
mod print;
mod tree;
mod utils;

use logos::Logos;
use node::*;
use parse::*;
use print::*;
use tree::*;

use std::{
  fmt::Debug,
  hint::unreachable_unchecked,
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
  fn link(&mut self, a: Node, b: Node) {
    match (a, b) {
      (Node::Era, Node::Era) => {}
      (Node::Era, Node::Auxiliary(r)) | (Node::Auxiliary(r), Node::Era) => unsafe {
        *r.0 = PackedNode::ERA
      },
      (Node::Era, Node::Principal(r)) | (Node::Principal(r), Node::Era) => {
        self.active.push((r, OwnedTree::era(r.kind())))
      }
      (Node::Principal(a), Node::Principal(b)) => self.active.push((a, b)),
      (Node::Principal(_), Node::Auxiliary(b)) => unsafe { *b.0 = a.pack() },
      (Node::Auxiliary(a), Node::Principal(_)) => unsafe { *a.0 = b.pack() },
      (Node::Auxiliary(aa), Node::Auxiliary(ba)) => unsafe {
        *aa.0 = b.pack();
        *ba.0 = a.pack();
      },
      _ => unsafe { unreachable_unchecked() },
    }
  }

  #[inline(always)]
  fn bind(&mut self, a: Node, b: OwnedTree) {
    match a {
      Node::Principal(a) => self.active.push((a, b)),
      Node::Auxiliary(a) => unsafe { *a.0 = Node::Principal(b).pack() },
      _ => unsafe { unreachable_unchecked() },
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
        .filter(|(_, x)| matches!(x, Node::Principal(..) | Node::Auxiliary(..)))
        .map(|(i, _)| (i, OwnedTree::clone(b))),
    );
    bv.extend(
      (0..b.tree().root().length())
        .map(|i| (i, b.tree().node(i)))
        .filter(|(_, x)| matches!(x, Node::Principal(..) | Node::Auxiliary(..)))
        .map(|(i, _)| (i, OwnedTree::clone(a))),
    );
    for &(ai, ref bc) in av.iter() {
      for &(bj, ref ac) in bv.iter() {
        self.link(
          Node::Auxiliary(ac.tree().offset(ai)),
          Node::Auxiliary(bc.tree().offset(bj)),
        )
      }
    }
    for (ai, b) in av.drain(..) {
      self.bind(a.tree().node(ai), b)
    }
    for (bi, a) in bv.drain(..) {
      self.bind(b.tree().node(bi), a)
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
          (Node::Era, Node::Ctr(_)) => {
            n += 2;
            a_era_stack += 2;
          }
          (Node::Ctr(_), Node::Era) => {
            n += 2;
            b_era_stack += 2
          }
          (Node::Ctr(_), Node::Ctr(_)) => n += 2,
          (r, Node::Ctr(l)) => {
            self.bind(r, OwnedTree::take(kind, b));
            b = b.offset(l - 1);
          }
          (Node::Ctr(l), r) => {
            self.bind(r, OwnedTree::take(kind, a));
            a = a.offset(l - 1);
          }
          (a, b) => self.link(a, b),
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
