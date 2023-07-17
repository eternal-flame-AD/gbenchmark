# gbenchmark

A benchmark library for Rust with inspiration from Go benchmarking.

## Concepts

A benchmark in gbenchmark consumes a **parameter**
(which may be how many iterations, how many threads, etc.)
and produces one **measure** (which may be time, memory allocated, etc.).
The **measure** can look at the parameter and last measure to decide whether to finish the benchmark or ask the **parameter** for more iterations.

## Creating a benchmark

To create a benchmark first declare the parameters and measures you want to use, then use `Benchmark::benchmark` to benchmark your function.

```rust
use std::time::Duration;
use gbenchmark::{
    measure::{TimeMeasure},
    Benchmark, RepetitionParams,
};

let bench = Benchmark::new(
    || RepetitionParams::default(),
    || TimeMeasure::with_min_time(Duration::from_millis(100)),
);
fn expensive_setup() {
    std::thread::sleep(Duration::from_millis(100));
}
fn do_something() {
    std::thread::sleep(Duration::from_millis(10));
}
let result = bench.benchmark(&mut |params, reset| {
    expensive_setup();
    reset();
    for _ in 0..params.nreps {
        do_something();
    }
});
assert!(result.measure.time - Duration::from_millis(9) < Duration::from_millis(2)); 
```


## Example

See [src/bin/gbenchmark_demo/main.rs](src/bin/gbenchmark_demo/main.rs) for a quick demo on getting a basic benchmark running.

```sh
> cargo run --bin gbenchmark_demo --release
   Compiling gbenchmark v0.1.0 (/home/yume/source/gbenchmark)
    Finished release [optimized] target(s) in 0.29s
     Running `target/release/gbenchmark_demo`
preallocated: 67108864 reps: 1.000 ns/op
push: 67108864 reps: 2.000 ns/op
preallocated: 1 reps: 0 allocs/op, 0 bytes alloc'ed/op
push: 1 reps: 6 allocs/op, 2016 bytes alloc'ed/op
```