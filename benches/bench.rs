#![feature(test)]

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use itrees::*;

extern crate test;

fn bench(c: &mut Criterion) {
  c.bench_function("test", |b| {
    b.iter_batched(
      || {
        parse_program(include_str!("../programs/dec_bits_comp.ic"))
          .unwrap()
          .1
      },
      |mut net| net.reduce(),
      BatchSize::SmallInput,
    )
  });
}

criterion_group!(benches, bench);
criterion_main!(benches);
