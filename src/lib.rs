#![feature(thread_local)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(new_uninit)]
#![feature(stmt_expr_attributes)]

mod net;
mod node;
mod parse;
mod print;
mod tree;
mod utils;

#[global_allocator]
#[thread_local]
static ALLOCATOR: BumpAlloc = BumpAlloc(UnsafeCell::new((None, 1 << 20)));

#[derive(Default)]
struct BumpAlloc(UnsafeCell<(Option<(*mut u8, usize)>, usize)>);

const MAX_CHUNK: usize = 1 << 30;

unsafe impl GlobalAlloc for BumpAlloc {
  unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
    let inner = unsafe { &mut *self.0.get() };
    if let Some((ptr, len)) = &mut inner.0 {
      let offset = ptr.align_offset(layout.align());
      let needed = offset + layout.size();
      if needed < *len {
        let r = ptr.offset(offset as isize);
        *ptr = ptr.offset(needed as isize);
        *len -= needed;
        return r;
      }
    }
    let len = (inner.1 << 1).min(MAX_CHUNK).max(layout.size());
    inner.1 = len;
    let alloc = System.alloc(Layout::from_size_align(len, layout.align()).unwrap());
    inner.0 = Some((alloc.offset(layout.size() as isize), len - layout.size()));
    alloc
  }

  unsafe fn dealloc(&self, _: *mut u8, _: std::alloc::Layout) {}
}

pub use net::*;
pub use node::*;
pub use parse::*;
pub use print::*;
pub use tree::*;

use std::{
  alloc::{GlobalAlloc, Layout, System},
  cell::UnsafeCell,
  env::args,
  fmt::Debug,
  fs,
  time::Instant,
};

#[allow(unused)]
fn main() {
  let path = args().nth(1).expect("must supply path");

  let program = fs::read_to_string(path).expect("invalid file");

  let (free, mut net) = parse_program(&program).unwrap();

  println!("{:?}", PrintNet(free, &net));

  net.reduce();

  println!("{:?}", PrintNet(free, &net));

  net.print_stats();
}
