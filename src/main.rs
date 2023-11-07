#![feature(new_uninit)]

mod r#ref;
mod tree;
mod utils;
mod word;

use r#ref::*;
use tree::*;
use word::*;

use std::fmt::Debug;

struct Net;

impl Net {
  fn link(&mut self, a: Ref, b: Ref) {
    dbg!(a, b);
    // todo!()
  }

  fn bind(&mut self, a: Ref, b: Tree) {
    dbg!(a, b.borrow());
    // todo!()
  }

  fn erase(&mut self, a: Ref) {
    dbg!(a);
    // todo!()
  }

  fn commute(&mut self, mut a: Tree, mut b: Tree) {
    for r in b.borrow_mut().refs {
      let x = Tree::from_form(a.kind, a.borrow().form);
      self.bind(std::mem::replace(r, Ref(x.raw as u64)), x);
    }
    for r in a.borrow_mut().refs {
      let x = Tree::from_form(b.kind, b.borrow().form);
      self.bind(std::mem::replace(r, Ref(x.raw as u64)), x);
    }
    for i in 0..a.borrow().refs.len() {
      for j in 0..b.borrow().refs.len() {
        self.link(
          UnpackedRef(j as u16 + 1, a.borrow().refs[i].0 as RawTree).pack(),
          UnpackedRef(i as u16 + 1, b.borrow().refs[j].0 as RawTree).pack(),
        );
      }
    }
    a.drop();
    b.drop();
  }

  fn annihilate(&mut self, mut a: Tree, mut b: Tree) {
    // todo!();
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
    UnpackedWord::Ctr(11).pack(),
    Word(4),
    UnpackedWord::Ctr(5).pack(),
    Word(3),
    Word::REF,
    UnpackedWord::Ctr(2).pack(),
    Word(2),
    Word::REF,
    Word::REF,
    UnpackedWord::Ctr(2).pack(),
    Word(1),
    Word::ERA,
    Word::REF,
  ];
  let a = Tree::from_form(0, data);
  let b = Tree::from_form(1, data);
  Net.commute(a, b);
  // // println!(
  // //   "{:#?}",
  // //   TreeRef {
  // //     kind: 0,
  // //     data,
  // //     refs: &[
  // //       UnpackedRef(0, std::ptr::null_mut()).pack(),
  // //       UnpackedRef(1, std::ptr::null_mut()).pack(),
  // //       UnpackedRef(2, std::ptr::null_mut()).pack(),
  // //       UnpackedRef(3, std::ptr::null_mut()).pack(),
  // //     ]
  // //   }
  // // );
}
