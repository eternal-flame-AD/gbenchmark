use std::{
    alloc::GlobalAlloc,
    fmt::Display,
    time::{Duration, Instant},
};

use crate::{
    alloc::{CountingAllocator, MemStats},
    Params,
};

/// Measure is the output of a benchmark.
pub trait Measure<P: Params>: Display {
    /// Returns true if the data is enough and no more measurements are needed.
    fn enough(&self, params: &P) -> bool;
    /// Observe runs the benchmark function and updates the measure.
    fn observe<F: FnOnce(&P, &mut dyn FnMut())>(&mut self, f: F, params: &P);
}

/// TimeMeasure measures the time it takes to run a benchmark, enforcing a minimum time.
pub struct TimeMeasure {
    min_time: Duration,
    start: Instant,
    /// Time per repetition.
    pub time: Duration,
    /// Total time.
    pub total_time: Duration,
}

impl TimeMeasure {
    /// Creates a new TimeMeasure with a minimum time of 1 second.
    pub fn new() -> Self {
        Self {
            min_time: Duration::from_secs(1),
            ..Default::default()
        }
    }
    /// Creates a new TimeMeasure with a minimum time of `min_time`.
    pub fn with_min_time(min_time: Duration) -> Self {
        Self {
            min_time,
            ..Default::default()
        }
    }
}

impl Display for TimeMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.time.as_secs_f64();
        if secs < 1e-6 {
            write!(f, "{:.3} ns/op", secs * 1e9)
        } else if secs < 1e-3 {
            write!(f, "{:.3} us/op", secs * 1e6)
        } else if secs < 1.0 {
            write!(f, "{:.3} ms/op", secs * 1e3)
        } else {
            write!(f, "{:.3} s/op", secs)
        }
    }
}

impl<P: Params> Measure<P> for TimeMeasure {
    fn enough(&self, _params: &P) -> bool {
        self.total_time >= self.min_time
    }
    fn observe<F: FnOnce(&P, &mut dyn FnMut())>(&mut self, f: F, params: &P) {
        self.start();
        f(params, &mut || self.start());
        self.stop();
        self.time /= params.nreps() as u32;
    }
}

impl TimeMeasure {
    fn start(&mut self) {
        self.start = Instant::now();
    }
    fn stop(&mut self) {
        self.total_time = self.start.elapsed();
        self.time += self.total_time;
    }
}

impl Default for TimeMeasure {
    fn default() -> Self {
        Self {
            min_time: Duration::from_secs(1),
            start: Instant::now(),
            time: Duration::default(),
            total_time: Duration::default(),
        }
    }
}

/// MemoryMeasure measures the memory allocated by a benchmark.
///
/// It takes a reference to a [CountingAllocator](crate::alloc::CountingAllocator) and
/// measures the memory allocated by it.
pub struct MemoryMeasure<Alloc: GlobalAlloc + 'static> {
    alloc: &'static CountingAllocator<Alloc>,
    start: MemStats,
    /// Memory allocations per repetition.
    pub memory: MemStats,
}

impl<Alloc: GlobalAlloc> MemoryMeasure<Alloc> {
    /// Creates a new MemoryMeasure.
    pub fn new(alloc: &'static CountingAllocator<Alloc>) -> Self {
        Self {
            alloc,
            start: MemStats::default(),
            memory: MemStats::default(),
        }
    }
    fn start(&mut self) {
        self.start = self.alloc.stats().clone();
    }
    fn stop(&mut self) {
        self.memory = self.alloc.stats().clone() - self.start.clone();
    }
}

impl<Alloc: GlobalAlloc, P: Params> Measure<P> for MemoryMeasure<Alloc> {
    fn enough(&self, _params: &P) -> bool {
        true
    }
    fn observe<F: FnOnce(&P, &mut dyn FnMut())>(&mut self, f: F, params: &P) {
        self.start();
        f(params, &mut || self.start());
        self.stop();
        self.memory = self.memory.clone() / params.nreps();
    }
}

impl<Alloc: GlobalAlloc> Display for MemoryMeasure<Alloc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} allocs/op, {} bytes alloc'ed/op",
            self.memory
                .alloc_count
                .load(std::sync::atomic::Ordering::Relaxed),
            self.memory
                .allocated
                .load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}
