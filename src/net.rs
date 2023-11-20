use crate::*;
use std::{hint::unreachable_unchecked, mem::size_of, time::Duration};

#[derive(Default, Debug)]
pub struct Net {
  pub(crate) active: Vec<(Tree, Tree)>,
  pub(crate) anni: usize,
  pub(crate) comm: usize,
  pub(crate) eras: usize,
  pub(crate) grft: usize,
  pub(crate) time: Duration,
  pub(crate) av: Vec<(Tree, usize, Result<Tree, usize>)>,
  pub(crate) at: Vec<PackedNode>,
  pub(crate) bv: Vec<(Tree, usize, Result<Tree, usize>)>,
  pub(crate) bt: Vec<PackedNode>,
}

impl Net {
  #[inline(always)]
  pub(crate) fn link(&mut self, a: Node, b: Node) {
    match (a, b) {
      (Node::Era, Node::Era) => {}
      (Node::Era, Node::Auxiliary(r)) | (Node::Auxiliary(r), Node::Era) => unsafe {
        *r.0 = PackedNode::ERA
      },
      (Node::Era, Node::Principal(r)) | (Node::Principal(r), Node::Era) => {
        self.active.push((r, Tree::NULL))
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
  pub(crate) fn bind(&mut self, a: Node, b: Tree) {
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
      if b.0 == Tree::NULL.0 {
        self.erase(a);
      } else if a.kind() == b.kind() {
        self.annihilate(a, b);
      } else {
        self.commute(a, b);
      }
    }
    self.time += start.elapsed();
  }

  pub fn print_stats(&self) {
    println!(
      "anni: {}; comm: {}; eras: {}; grft: {}; time: {:.2?}",
      self.anni, self.comm, self.eras, self.grft, self.time
    );
  }

  #[inline(never)]
  pub(crate) fn commute(&mut self, a: Tree, b: Tree) {
    self.comm += 1;
    let mut av = std::mem::take(&mut self.av);
    let mut bv = std::mem::take(&mut self.bv);
    let a_len = _commute_scan(&mut self.grft, &mut self.at, a, &mut av);
    let b_len = _commute_scan(&mut self.grft, &mut self.bt, b, &mut bv);
    _commute_copy(a_len == self.at.len(), &mut self.bt, &mut av, a_len, a);
    _commute_copy(b_len == self.bt.len(), &mut self.at, &mut bv, b_len, b);
    self.at.clear();
    self.bt.clear();
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
        self.bind(aa.node(), bc)
      }
    }
    for &(ba, _, ac) in &bv {
      if let Ok(ac) = ac {
        self.bind(ba.node(), ac)
      }
    }
    av.clear();
    bv.clear();
    self.av = av;
    self.bv = bv;

    fn _commute_scan(
      g: &mut usize,
      t: &mut Vec<PackedNode>,
      mut a: Tree,
      av: &mut Vec<(Tree, usize, Result<Tree, usize>)>,
    ) -> usize {
      let mut a_len = 1;
      let mut i = 0;
      let kind = a.kind();
      while i < a_len {
        let node = a.node();
        match node {
          Node::Principal(mut p) if p.kind() == kind => {
            *g += 1;
            let mut n = 1;
            while n > 0 {
              let node = p.node();
              match node {
                Node::Ctr(..) => n += 2,
                Node::Era => {}
                _ => av.push((p, t.len(), Ok(Tree::NULL))),
              }
              t.push(node.pack());
              n -= 1;
              p = p.offset(1);
            }
            i += 1;
            a = a.offset(1);
            continue;
          }
          Node::Ctr(..) => a_len += 2,
          Node::Era => {}
          _ => av.push((a, t.len(), Ok(Tree::NULL))),
        }
        t.push(node.pack());
        i += 1;
        a = a.offset(1);
      }
      a_len
    }

    fn _commute_copy(
      x: bool,
      bt: &mut Vec<PackedNode>,
      av: &mut Vec<(Tree, usize, Result<Tree, usize>)>,
      a_len: usize,
      a: Tree,
    ) {
      for (aa, _, bc) in av.iter_mut() {
        if x {
          if let Node::Auxiliary(t) = aa.node() {
            if (a.0 as usize..a.0 as usize + a_len * size_of::<usize>()).contains(&(t.0 as usize)) {
              *bc = Err((t.0 as usize - a.0 as usize) / size_of::<usize>());
              continue;
            }
          }
        }
        let len = bt.len();
        *bc = Ok(Tree::clone(Tree(&mut bt[0] as *mut _), len));
      }
    }
  }

  pub(crate) fn erase(&mut self, mut a: Tree) {
    self.eras += 1;
    let mut n = 1;
    while n > 0 {
      match a.node() {
        Node::Ctr(..) => {
          n += 1;
        }
        x => {
          self.link(x, Node::Era);
          n -= 1;
        }
      }
      a = a.offset(1);
    }
  }

  #[inline(never)]
  pub(crate) fn annihilate(&mut self, mut a: Tree, mut b: Tree) {
    self.anni += 1;
    let mut n = 1usize;
    while n > 0 {
      match (a.node(), b.node()) {
        (Node::Era, Node::Ctr(..)) => {
          let mut n = 2;
          while n > 0 {
            b = b.offset(1);
            match b.node() {
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
            match a.node() {
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
            match b.node() {
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
            match a.node() {
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
