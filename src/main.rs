#![feature(new_uninit)]

mod r#ref;
mod tree;
mod utils;
mod word;

use r#ref::*;
use tree::*;
use word::*;

use std::fmt::Debug;

#[derive(Default)]
struct Net {
  scratch: Vec<u64>,
}

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
    let mut cur_a = TreeRange::FULL;
    let mut cur_b = TreeRange::FULL;
    {
      match (a.slice(cur_a).unpack_node(), b.slice(cur_b).unpack_node()) {
        (UnpackedTreeNode::Era, UnpackedTreeNode::Era) => {}
        (UnpackedTreeNode::Era, UnpackedTreeNode::Ref(r)) => self.erase(r),
        (UnpackedTreeNode::Ref(r), UnpackedTreeNode::Era) => self.erase(r),
        (UnpackedTreeNode::Ref(a), UnpackedTreeNode::Ref(b)) => self.link(a, b),
        (UnpackedTreeNode::Era, UnpackedTreeNode::Ctr(left, right)) => {
          self._annihilate_stack_push(cur_a, right);
          self._annihilate_stack_push(cur_a, left);
        }
        (UnpackedTreeNode::Ctr(left, right), UnpackedTreeNode::Era) => {
          self._annihilate_stack_push(right, cur_b);
          self._annihilate_stack_push(left, cur_a);
        }
        (UnpackedTreeNode::Ctr(a_left, a_right), UnpackedTreeNode::Ctr(b_left, b_right)) => {
          self._annihilate_stack_push(a_right, b_right);
          self._annihilate_stack_push(a_left, b_left);
        }
        (UnpackedTreeNode::Ref(_), UnpackedTreeNode::Ctr(left, right)) => todo!(),
        (UnpackedTreeNode::Ctr(left, right), UnpackedTreeNode::Ref(_)) => todo!(),
        (_, UnpackedTreeNode::Cup(_)) => todo!(),
        (UnpackedTreeNode::Cup(_), _) => todo!(),
      }
    }
  }

  fn _annihilate_stack_push(&mut self, a: TreeRange, b: TreeRange) {
    // self.scratch.push(bytemuck::must_cast(a));
    // self.scratch.push(bytemuck::must_cast(b));
  }
}

fn main() {
  let data = &[
    UnpackedWord::Ctr(Dimensions {
      refs_len: 4,
      form_len: 9,
    })
    .pack(),
    UnpackedWord::Ctr(Dimensions {
      refs_len: 3,
      form_len: 5,
    })
    .pack(),
    Word::REF,
    UnpackedWord::Ctr(Dimensions {
      refs_len: 2,
      form_len: 3,
    })
    .pack(),
    Word::REF,
    Word::REF,
    UnpackedWord::Ctr(Dimensions {
      refs_len: 1,
      form_len: 3,
    })
    .pack(),
    Word::ERA,
    Word::REF,
  ];
  let a = Tree::from_form(0, data);
  let b = Tree::from_form(1, data);
  Net::default().annihilate(a, b);
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
