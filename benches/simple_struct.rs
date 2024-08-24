use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{random, Rng};
use sov::StructOfVecs;

#[derive(Clone, StructOfVecs)]
struct Foo {
    x: u64,
    y: u8,
}

fn random_vec(len: usize) -> (Vec<Foo>, VecFoo) {
    let mut vec_foo = VecFoo::with_capacity(len);
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        let foo = Foo {
            x: random(),
            y: random(),
        };
        vec_foo.push(foo.clone());
        vec.push(foo);
    }

    (vec, vec_foo)
}

pub fn bench_sum_u64s(c: &mut Criterion) {
    let (vec, vec_foo) = random_vec(100_000);

    let mut group = c.benchmark_group("sum 100,000 u64s");

    group.bench_with_input(BenchmarkId::new("u64s", "sov"), &vec_foo, |b, input| {
        b.iter(|| {
            let sum: u64 = black_box(input).xs().iter().copied().sum();
            black_box(sum);
        })
    });

    group.bench_with_input(BenchmarkId::new("u64s", "naive"), &vec, |b, input| {
        b.iter(|| {
            let sum: u64 = black_box(input).iter().map(|foo| foo.x).sum();
            black_box(sum);
        })
    });

    group.bench_with_input(BenchmarkId::new("u8s", "sov"), &vec_foo, |b, input| {
        b.iter(|| {
            let sum: u8 = black_box(input).ys().iter().copied().sum();
            black_box(sum);
        })
    });

    group.bench_with_input(BenchmarkId::new("u8s", "naive"), &vec, |b, input| {
        b.iter(|| {
            let sum: u8 = black_box(input).iter().map(|foo| foo.y).sum();
            black_box(sum);
        })
    });
}

criterion_group!(benches, bench_sum_u64s);
criterion_main!(benches);
