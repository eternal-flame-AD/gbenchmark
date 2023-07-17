use std::{alloc::System, hint::black_box, time::Duration};

use gbenchmark::{
    alloc::CountingAllocator,
    measure::{MemoryMeasure, TimeMeasure},
    Benchmark, RepetitionParams,
};

fn push_numbers(v: &mut Vec<usize>, n: usize) {
    for i in 0..n {
        v.push(i);
    }
}

fn benchmark_time() {
    let bench = Benchmark::new(
        || RepetitionParams::default(),
        || TimeMeasure::with_min_time(Duration::from_millis(100)),
    );
    let result_preallocated = bench.benchmark(&mut |params, reset| {
        let mut v = Vec::with_capacity(params.nreps);
        reset();
        push_numbers(&mut v, params.nreps);
        black_box(v);
    });
    let result_push = bench.benchmark(&mut |params, reset| {
        let mut v = Vec::new();
        reset();
        push_numbers(&mut v, params.nreps);
        black_box(v);
    });
    println!("preallocated: {}", result_preallocated);
    println!("push: {}", result_push);
}

// for memory benchmarks we need to hook into the allocator
#[global_allocator]
static ALLOC: CountingAllocator<System> = gbenchmark::alloc::CountingAllocator::new(System);

fn benchmark_allocations() {
    let bench = Benchmark::new(
        || RepetitionParams::default(),
        || MemoryMeasure::new(&ALLOC),
    );
    let result_preallocated = bench.benchmark(&mut |params, reset| {
        let mut v = Vec::with_capacity(params.nreps * 100);
        reset();
        for _i in 0..params.nreps {
            push_numbers(&mut v, 100);
        }
        black_box(v);
    });
    let result_push = bench.benchmark(&mut |params, reset| {
        let mut v = Vec::new();
        reset();
        for _i in 0..params.nreps {
            push_numbers(&mut v, 100);
        }
        black_box(v);
    });
    println!("preallocated: {}", result_preallocated);
    println!("push: {}", result_push);
}

fn main() {
    benchmark_time();
    benchmark_allocations();
}
