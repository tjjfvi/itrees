#![feature(test)]

use std::fs;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use itrees::*;

extern crate test;

fn bench(c: &mut Criterion) {
  for path in &[
    "programs/dec_bits.ic",
    "programs/dec_bits_comp.ic",
    "programs/dec_bits_tree.ic",
    "programs/dec_bits_tree_comp.ic",
  ] {
    let file = fs::read_to_string(path).expect("invalid file");
    let mut first = true;
    c.bench_function(path, |b| {
      b.iter_batched(
        || parse_program(&file).unwrap().1,
        |mut net| {
          net.reduce();
          if first {
            first = false;
            println!();
            net.print_stats();
          }
        },
        BatchSize::SmallInput,
      )
    });
  }
}

criterion_group!(benches, bench);
criterion_main!(benches);
