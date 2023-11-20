use crate::*;
use std::{hint::unreachable_unchecked, mem::size_of, time::Duration};

#[derive(Default, Debug)]
pub struct Net {
  pub active: Vec<(Tree, Tree)>,
  pub anni: usize,
  pub comm: usize,
  pub grft: usize,
  pub time: Duration,
  pub av: Vec<(usize, Result<Tree, usize>)>,
  pub at: Vec<PackedNode>,
  pub bv: Vec<(usize, Result<Tree, usize>)>,
  pub bt: Vec<PackedNode>,
}

impl Net {
  #[inline(always)]
  pub fn link(&mut self, a: Node, b: Node) {
    match (a, b) {
      (Node::Era, Node::Era) => {}
      (Node::Era, Node::Auxiliary(r)) | (Node::Auxiliary(r), Node::Era) => unsafe {
        *r.0 = PackedNode::ERA
      },
      (Node::Era, Node::Principal(r)) | (Node::Principal(r), Node::Era) => {
        self.active.push((r, Tree::ERA))
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
  pub fn bind(&mut self, a: Node, b: Tree) {
    match a {
      Node::Principal(a) => self.active.push((a, b)),
      Node::Auxiliary(a) => unsafe { *a.0 = Node::Principal(b).pack() },
      _ => unsafe { unreachable_unchecked() },
    }
  }

  #[inline(never)]
  pub fn reduce(&mut self) {
    let start = Instant::now();
    while let Some((a, b)) = self.active.pop() {
      if a.kind().is_none() || b.kind().is_none() || a.kind() == b.kind() {
        self.annihilate(a, b);
      } else {
        self.commute(a, b);
      }
    }
    self.time += start.elapsed();
  }

  pub fn print_stats(&self) {
    println!(
      "anni: {}; comm: {}; grft: {}; time: {:.2?}",
      self.anni, self.comm, self.grft, self.time
    );
  }

  #[inline(never)]
  pub fn commute(&mut self, a: Tree, b: Tree) {
    self.comm += 1;
    let mut av = std::mem::take(&mut self.av);
    let mut bv = std::mem::take(&mut self.bv);
    av.reserve(a.root().length() / 2 + 1);
    for i in 0..a.root().length() {
      let node = a.node(i);
      match node {
        Node::Auxiliary(t) if a.contains(t) => {
          av.push((i, Err((t.0 as usize - a.0 as usize) / size_of::<usize>())))
        }
        Node::Auxiliary(_) | Node::Principal(_) => {
          av.push((i, Ok(Tree::clone(b))));
        }
        _ => {}
      }
    }
    bv.reserve(b.root().length() / 2 + 1);
    for i in 0..b.root().length() {
      let node = b.node(i);
      match node {
        Node::Auxiliary(t) if b.contains(t) => {
          bv.push((i, Err((t.0 as usize - b.0 as usize) / size_of::<usize>())))
        }
        Node::Auxiliary(_) | Node::Principal(_) => {
          bv.push((i, Ok(Tree::clone(a))));
        }
        _ => {}
      }
    }
    for &(ai, bc) in av.iter() {
      for &(bj, ac) in bv.iter() {
        match (ac, bc) {
          (Ok(ac), Ok(bc)) => self.link(
            Node::Auxiliary(ac.offset(ai)),
            Node::Auxiliary(bc.offset(bj)),
          ),
          (Ok(ac), Err(i)) => unsafe { *ac.offset(ai).0 = Node::Auxiliary(ac.offset(i)).pack() },
          (Err(i), Ok(bc)) => unsafe { *bc.offset(bj).0 = Node::Auxiliary(bc.offset(i)).pack() },
          _ => {}
        }
      }
    }
    for &(ai, bc) in &av {
      if let Ok(bc) = bc {
        self.bind(a.node(ai), bc)
      }
    }
    for &(bi, ac) in &bv {
      if let Ok(ac) = ac {
        self.bind(b.node(bi), ac)
      }
    }
    av.clear();
    bv.clear();
    self.av = av;
    self.bv = bv;
  }

  #[inline(never)]
  pub fn annihilate(&mut self, mut a: Tree, mut b: Tree) {
    self.anni += 1;
    let mut n = 1usize;
    let mut a_era_stack = 0usize;
    let mut b_era_stack = 0usize;
    while n > 0 {
      match (a.root(), b.root()) {
        (Node::Era, Node::Ctr(..)) => {
          n += 2;
          a_era_stack += 2;
        }
        (Node::Ctr(..), Node::Era) => {
          n += 2;
          b_era_stack += 2
        }
        (Node::Ctr(..), Node::Ctr(..)) => n += 2,
        (r, Node::Ctr(l, _)) => {
          self.bind(r, b);
          b = b.offset(l - 1);
        }
        (Node::Ctr(l, _), r) => {
          self.bind(r, a);
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
}
