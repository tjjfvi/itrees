use crate::*;
use std::{hint::unreachable_unchecked, time::Duration};

#[derive(Default, Debug)]
pub struct Net {
  pub active: Vec<(Tree, Tree)>,
  pub anni: usize,
  pub comm: usize,
  pub grft: usize,
  pub time: Duration,
  pub av: Vec<(Tree, usize, Result<Tree, usize>)>,
  pub at: Vec<PackedNode>,
  pub bv: Vec<(Tree, usize, Result<Tree, usize>)>,
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
    let mut a_len = 1;
    let mut i = 0;
    while i < a_len {
      let node = a.node(i);
      match node {
        // Node::Auxiliary(t) if a.contains(t) => av.push((
        //   node.pack(),
        //   i,
        //   Err((t.0 as usize - a.0 as usize) / size_of::<usize>()),
        // )),
        // Node::Principal(p) if p.kind() == a.kind() => {
        //   self.g += 1;
        //   todo!()
        // }
        Node::Auxiliary(_) | Node::Principal(_) => {
          av.push((a.offset(i), i, Ok(Tree::NULL)));
        }
        Node::Ctr(..) => {
          a_len += 2;
        }
        _ => {}
      }
      i += 1;
    }
    let mut b_len = 1;
    let mut i = 0;
    while i < b_len {
      // dbg!(i);
      let node = b.node(i);
      match node {
        // Node::Auxiliary(t) if b.contains(t) => bv.push((
        //   node.pack(),
        //   i,
        //   Err((t.0 as usize - b.0 as usize) / size_of::<usize>()),
        // )),
        Node::Auxiliary(_) | Node::Principal(_) => {
          bv.push((b.offset(i), i, Ok(Tree::NULL)));
        }
        Node::Ctr(..) => {
          b_len += 2;
        }
        _ => {}
      }
      i += 1;
    }
    for (_, _, bc) in av.iter_mut() {
      if let Ok(bc) = bc {
        *bc = Tree::clone(b, b_len);
      }
    }
    for (_, _, ac) in bv.iter_mut() {
      if let Ok(ac) = ac {
        *ac = Tree::clone(a, a_len);
      }
    }
    for &(_, ai, bc) in av.iter() {
      for &(_, bj, ac) in bv.iter() {
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
    for &(aa, _, bc) in &av {
      if let Ok(bc) = bc {
        self.bind(aa.root(), bc)
      }
    }
    for &(ba, _, ac) in &bv {
      if let Ok(ac) = ac {
        self.bind(ba.root(), ac)
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
    while n > 0 {
      match (a.root(), b.root()) {
        (Node::Era, Node::Ctr(..)) => {
          let mut n = 2;
          while n > 0 {
            b = b.offset(1);
            match b.root() {
              Node::Ctr(..) => {
                n += 1;
              }
              x => {
                self.link(x, Node::Era);
                n -= 1;
              }
            }
          }
        }
        (Node::Ctr(..), Node::Era) => {
          let mut n = 2;
          while n > 0 {
            a = a.offset(1);
            match a.root() {
              Node::Ctr(..) => {
                n += 1;
              }
              x => {
                self.link(x, Node::Era);
                n -= 1;
              }
            }
          }
        }
        (Node::Ctr(..), Node::Ctr(..)) => n += 2,
        (r, Node::Ctr(_)) => {
          self.bind(r, b);
          let mut n = 2;
          while n > 0 {
            b = b.offset(1);
            match b.root() {
              Node::Ctr(_) => {
                n += 1;
              }
              _ => {
                n -= 1;
              }
            }
          }
        }
        (Node::Ctr(_), r) => {
          self.bind(r, a);
          let mut n = 2;
          while n > 0 {
            a = a.offset(1);
            match a.root() {
              Node::Ctr(_) => {
                n += 1;
              }
              _ => {
                n -= 1;
              }
            }
          }
        }
        (a, b) => self.link(a, b),
      }
      a = a.offset(1);
      b = b.offset(1);
      n -= 1;
    }
  }
}
