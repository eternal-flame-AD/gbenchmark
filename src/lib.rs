//! # gbenchmark
//!
//! A benchmark library for Rust.
//!
//! ## Concepts
//!
//!
//! A benchmark in gbenchmark consumes a **parameter**
//! (which may be how many iterations, how many threads, etc.)
//! and produces one **measure** (which may be time, memory allocated, etc.).
//! The **measure** can look at the parameter and last measure to decide whether to finish the benchmark or ask the **parameter** for more iterations.
//!
//! ## Creating a benchmark
//!
//! To create a benchmark first declare the parameters and measures you want to use, then use `Benchmark::benchmark` to benchmark your function.
//!
//! ```rust
//! use std::time::Duration;
//! use gbenchmark::{
//!     measure::{TimeMeasure},
//!     Benchmark, RepetitionParams,
//! };
//!
//! let bench = Benchmark::new(
//!     || RepetitionParams::default(),
//!     || TimeMeasure::with_min_time(Duration::from_millis(100)),
//! );
//! fn expensive_setup() {
//!     std::thread::sleep(Duration::from_millis(100));
//! }
//! fn do_something() {
//!     std::thread::sleep(Duration::from_millis(10));
//! }
//! let result = bench.benchmark(&mut |params, reset| {
//!     expensive_setup();
//!     reset();
//!     for _ in 0..params.nreps {
//!         do_something();
//!     }
//! });
//! assert!(result.measure.time - Duration::from_millis(9) < Duration::from_millis(2));
//! ```
//!
#![warn(missing_docs)]

use std::fmt::Display;

use measure::Measure;

/// The [alloc] module contains an allocator wrapper that counts allocations.
pub mod alloc;
/// The [measure] module contains the `Measure` trait and some implementations.
pub mod measure;

/// Final parameters and measure of a benchmark.
pub struct BenchmarkResult<P: Params, M: Measure<P>> {
    /// The parameters used for the benchmark.
    pub params: P,
    /// The measure of the benchmark.
    pub measure: M,
}

impl<P: Params, M: Measure<P>> Display for BenchmarkResult<P, M>
where
    P: Display,
    M: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.params, self.measure)
    }
}

/// A Benchmark system,
///
/// It holds a factory for the parameters and a factory for the measure,
/// which will be used to initialize the parameters and measure for each benchmark.
pub struct Benchmark<P: Params, PF: Fn() -> P, M: Measure<P>, MF: Fn() -> M> {
    params_factory: PF,
    measure_factory: MF,
}

impl<P, PF, M, MF> Benchmark<P, PF, M, MF>
where
    P: Params,
    PF: Fn() -> P,
    M: Measure<P>,
    MF: Fn() -> M,
{
    /// Create a new benchmark.
    pub fn new(params_factory: PF, measure_factory: MF) -> Self {
        Self {
            params_factory,
            measure_factory,
        }
    }
    /// Benchmark a function to [Measure]'s satisfaction.
    pub fn benchmark<F: FnMut(&P, &mut dyn FnMut())>(&self, f: &mut F) -> BenchmarkResult<P, M> {
        let mut params = (self.params_factory)();
        loop {
            let mut result = (self.measure_factory)();
            result.observe(&mut *f, &params);
            if result.enough(&params) {
                break BenchmarkResult {
                    params,
                    measure: result,
                };
            }
            params.more();
        }
    }
}

/// Trait for benchmark parameters.
pub trait Params {
    /// Number of repetitions in total.
    fn nreps(&self) -> usize;
    /// [Measure] asked for more repetitions.
    fn more(&mut self);
}

/// A simple parameter that doubles the number of repetitions each time.
pub struct RepetitionParams {
    /// Number of repetitions in total.
    pub nreps: usize,
}

impl Display for RepetitionParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} reps", self.nreps)
    }
}

impl Default for RepetitionParams {
    fn default() -> Self {
        Self { nreps: 1 }
    }
}

impl Params for RepetitionParams {
    fn nreps(&self) -> usize {
        self.nreps
    }
    fn more(&mut self) {
        self.nreps *= 2;
    }
}
