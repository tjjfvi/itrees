use crate::*;
use std::{hint::unreachable_unchecked, mem::size_of};

#[derive(Default, Debug)]
pub struct Net {
  pub active: Vec<(OwnedTree, OwnedTree)>,
  pub a: usize,
  pub c: usize,
  pub av: Vec<(usize, Result<OwnedTree, usize>)>,
  pub bv: Vec<(usize, Result<OwnedTree, usize>)>,
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
  pub fn bind(&mut self, a: Node, b: OwnedTree) {
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
    a.drop();
    b.drop();
    Some(())
  }

  #[inline(never)]
  pub fn reduce(&mut self) -> i32 {
    let mut n = 0;
    while let Some(_) = self.reduce_one() {
      n += 1;
    }
    println!("{} ann, {} com", self.a, self.c);
    n
  }

  #[inline(never)]
  pub fn commute(&mut self, a: OwnedTree, b: OwnedTree) {
    self.c += 1;
    let mut av = std::mem::take(&mut self.av);
    let mut bv = std::mem::take(&mut self.bv);
    av.reserve(a.tree().root().length() / 2 + 1);
    for i in 0..a.tree().root().length() {
      let node = a.tree().node(i);
      match node {
        Node::Auxiliary(t) if a.tree().contains(t) => av.push((
          i,
          Err((t.0 as usize - a.tree().0 as usize) / size_of::<usize>()),
        )),
        Node::Auxiliary(_) | Node::Principal(_) => {
          av.push((i, Ok(OwnedTree::clone(b))));
        }
        _ => {}
      }
    }
    for i in 0..b.tree().root().length() {
      let node = b.tree().node(i);
      match node {
        Node::Auxiliary(t) if b.tree().contains(t) => bv.push((
          i,
          Err((t.0 as usize - b.tree().0 as usize) / size_of::<usize>()),
        )),
        Node::Auxiliary(_) | Node::Principal(_) => {
          bv.push((i, Ok(OwnedTree::clone(a))));
        }
        _ => {}
      }
    }
    for &(ai, bc) in av.iter() {
      for &(bj, ac) in bv.iter() {
        match (ac, bc) {
          (Ok(ac), Ok(bc)) => self.link(
            Node::Auxiliary(ac.tree().offset(ai)),
            Node::Auxiliary(bc.tree().offset(bj)),
          ),
          (Ok(ac), Err(i)) => unsafe {
            *ac.tree().offset(ai).0 = Node::Auxiliary(ac.tree().offset(i)).pack()
          },
          (Err(i), Ok(bc)) => unsafe {
            *bc.tree().offset(bj).0 = Node::Auxiliary(bc.tree().offset(i)).pack()
          },
          _ => {}
        }
      }
    }
    for (ai, bc) in av.drain(..) {
      if let Ok(bc) = bc {
        self.bind(a.tree().node(ai), bc)
      }
    }
    for (bi, ac) in bv.drain(..) {
      if let Ok(ac) = ac {
        self.bind(b.tree().node(bi), ac)
      }
    }
    self.av = av;
    self.bv = bv;
  }

  #[inline(never)]
  pub fn annihilate(&mut self, a: OwnedTree, b: OwnedTree) {
    self.a += 1;
    let kind = a.kind();
    let mut a = a.tree();
    let mut b = b.tree();
    let mut n = 1usize;
    let mut a_era_stack = 0usize;
    let mut b_era_stack = 0usize;
    while n > 0 {
      match (a.root(), b.root()) {
        (Node::Era, Node::Ctr(_, _)) => {
          n += 2;
          a_era_stack += 2;
        }
        (Node::Ctr(_, _), Node::Era) => {
          n += 2;
          b_era_stack += 2
        }
        (Node::Ctr(_, _), Node::Ctr(_, _)) => n += 2,
        (r, Node::Ctr(l, _)) => {
          self.bind(r, OwnedTree::take(kind, b));
          b = b.offset(l - 1);
        }
        (Node::Ctr(l, _), r) => {
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
}
