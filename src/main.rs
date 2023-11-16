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

use std::fmt::Debug;

#[derive(Default, Debug)]
struct Net {
  active: Vec<(RawTree, RawTree)>,
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
      UnpackedRef::Principal(a) => self
        .active
        .push((a, OwnedTree::new(unsafe { *a }, &[Word::ERA]).raw)),
    }
  }

  pub fn reduce_one(&mut self) -> Option<()> {
    let (a, b) = self.active.pop()?;
    let (a, b) = unsafe { (OwnedTree::from_raw(a), OwnedTree::from_raw(b)) };
    if a.kind == b.kind {
      self.annihilate(a, b);
    } else {
      self.commute(a, b);
    }
    Some(())
  }

  fn commute(&mut self, a: OwnedTree, b: OwnedTree) {
    let a_indices = a
      .iter()
      .enumerate()
      .filter(|(_, x)| matches!(x.unpack(), UnpackedWord::Ref(_)))
      .map(|(i, _)| i)
      .collect::<Vec<_>>();
    let b_indices = b
      .iter()
      .enumerate()
      .filter(|(_, x)| matches!(x.unpack(), UnpackedWord::Ref(_)))
      .map(|(i, _)| i)
      .collect::<Vec<_>>();
    let mut a_clones = b_indices
      .iter()
      .map(|_| OwnedTree::new(a.kind, &*a))
      .collect::<Vec<_>>();
    let mut b_clones = a_indices
      .iter()
      .map(|_| OwnedTree::new(b.kind, &*b))
      .collect::<Vec<_>>();
    for (i, ai) in a_indices.iter().copied().enumerate() {
      for (j, bj) in b_indices.iter().copied().enumerate() {
        self.link(
          UnpackedRef::Auxiliary(&mut a_clones[j][ai] as *mut _ as *mut _).pack(),
          UnpackedRef::Auxiliary(&mut b_clones[i][bj] as *mut _ as *mut _).pack(),
        )
      }
    }
    for (ai, b) in a_indices.iter().copied().zip(b_clones) {
      self.bind(Ref(a[ai].0), b)
    }
    for (bi, a) in b_indices.iter().copied().zip(a_clones) {
      self.bind(Ref(b[bi].0), a)
    }
    a.drop();
    b.drop();
  }

  fn annihilate(&mut self, a: OwnedTree, b: OwnedTree) {
    let kind = a.kind;
    {
      let mut ai = 0;
      let mut bi = 0;
      let mut a_era_stack = 0;
      let mut b_era_stack = 0;
      while ai < a.len() {
        match (a[ai].unpack(), b[bi].unpack()) {
          (UnpackedWord::Era, UnpackedWord::Era) => {}
          (UnpackedWord::Era, UnpackedWord::Ref(r)) => self.erase(r),
          (UnpackedWord::Ref(r), UnpackedWord::Era) => self.erase(r),
          (UnpackedWord::Ref(a), UnpackedWord::Ref(b)) => self.link(a, b),
          (UnpackedWord::Era, UnpackedWord::Ctr(_)) => a_era_stack += 2,
          (UnpackedWord::Ctr(_), UnpackedWord::Era) => b_era_stack += 2,
          (UnpackedWord::Ctr(_), UnpackedWord::Ctr(_)) => {}
          (UnpackedWord::Ref(r), UnpackedWord::Ctr(l)) => {
            self.bind(r, OwnedTree::take(kind, &b[bi..]));
            bi += l - 1;
          }
          (UnpackedWord::Ctr(l), UnpackedWord::Ref(r)) => {
            self.bind(r, OwnedTree::take(kind, &a[ai..]));
            ai += l - 1;
          }
        }
        if a_era_stack != 0 {
          a_era_stack -= 1;
        } else {
          ai += 1;
        }
        if b_era_stack != 0 {
          b_era_stack -= 1;
        } else {
          bi += 1;
        }
      }
    }
    a.drop();
    b.drop();
  }
}

fn main() {
  let mut foo = Token::lexer(
    "

    out

    add = (
      (((z i0) o0) ((o0 i1) o1))
      ((z [i0 i1]) o1)
    )
    
    one = ((z (z o)) o)
    
    [one0 [one1 one2]] = one
    {2 add0 add1} = add
    
    add0 = ((one0 one1) two)
    add1 = ((two one2) three)
    
    out = three
    
    
",
  );

  unsafe {
    let (a, mut b) = parse_program(&mut foo).unwrap();

    println!("{:?}", PrintNet(&*a, &b));

    let mut n = 0;
    while let Some(_) = b.reduce_one() {
      n += 1;
      println!("{:?}", PrintNet(&*a, &b));
    }

    println!("{} steps\n", n);

    println!("{:?}", PrintNet(&*a, &b));
  }

  // let data = &[
  //   UnpackedWord::Ctr(Dimensions {
  //     refs_len: 4,
  //     form_len: 9,
  //   })
  //   .pack(),
  //   UnpackedWord::Ctr(Dimensions {
  //     refs_len: 3,
  //     form_len: 5,
  //   })
  //   .pack(),
  //   Word::REF,
  //   UnpackedWord::Ctr(Dimensions {
  //     refs_len: 2,
  //     form_len: 3,
  //   })
  //   .pack(),
  //   Word::REF,
  //   Word::REF,
  //   UnpackedWord::Ctr(Dimensions {
  //     refs_len: 1,
  //     form_len: 3,
  //   })
  //   .pack(),
  //   Word::ERA,
  //   Word::REF,
  // ];
  // let a = Tree::from_form(0, data);
  // let b = Tree::from_form(1, data);
  // Net::default().annihilate(a, b);
  // // // println!(
  // // //   "{:#?}",
  // // //   TreeRef {
  // // //     kind: 0,
  // // //     data,
  // // //     refs: &[
  // // //       UnpackedRef(0, std::ptr::null_mut()).pack(),
  // // //       UnpackedRef(1, std::ptr::null_mut()).pack(),
  // // //       UnpackedRef(2, std::ptr::null_mut()).pack(),
  // // //       UnpackedRef(3, std::ptr::null_mut()).pack(),
  // // //     ]
  // // //   }
  // // // );
}
