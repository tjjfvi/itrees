#![feature(new_uninit)]

mod net;
mod node;
mod parse;
mod print;
mod tree;
mod utils;

pub use net::*;
pub use node::*;
pub use parse::*;
pub use print::*;
pub use tree::*;

use std::fmt::Debug;

#[allow(unused)]
fn main() {
  let program = include_str!("../programs/dec_bits_comp.ic");

  let (a, mut b) = parse_program(program).unwrap();

  unsafe {
    println!("{:?}", PrintNet(&*a, &b));
  }

  println!("{} steps", b.reduce());

  unsafe {
    println!("{:?}", PrintNet(&*a, &b));
  }
}
