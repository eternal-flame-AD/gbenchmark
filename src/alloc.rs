use std::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

/// Statistics about memory allocations.
#[derive(Debug, Default)]
pub struct MemStats {
    /// Number of allocations.
    pub alloc_count: AtomicUsize,
    /// Number of bytes allocated.
    pub allocated: AtomicUsize,
}

impl Clone for MemStats {
    fn clone(&self) -> Self {
        Self {
            alloc_count: AtomicUsize::new(self.alloc_count.load(Ordering::Relaxed)),
            allocated: AtomicUsize::new(self.allocated.load(Ordering::Relaxed)),
        }
    }
}

impl std::ops::Div<usize> for MemStats {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self {
            alloc_count: AtomicUsize::new(self.alloc_count.load(Ordering::Relaxed) / rhs),
            allocated: AtomicUsize::new(self.allocated.load(Ordering::Relaxed) / rhs),
        }
    }
}

impl std::ops::Sub for MemStats {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            alloc_count: AtomicUsize::new(
                self.alloc_count.load(Ordering::Relaxed) - rhs.alloc_count.load(Ordering::Relaxed),
            ),
            allocated: AtomicUsize::new(
                self.allocated.load(Ordering::Relaxed) - rhs.allocated.load(Ordering::Relaxed),
            ),
        }
    }
}

/// A global allocator wrapper that counts the number of allocations and the number of bytes
pub struct CountingAllocator<Alloc: GlobalAlloc> {
    inner: Alloc,
    stats: MemStats,
}

impl<Alloc: GlobalAlloc> CountingAllocator<Alloc> {
    /// Creates a new CountingAllocator.
    pub const fn new(inner: Alloc) -> Self {
        Self {
            inner,
            stats: MemStats {
                alloc_count: AtomicUsize::new(0),
                allocated: AtomicUsize::new(0),
            },
        }
    }
    /// Returns the statistics about memory allocations.
    pub fn stats(&self) -> &MemStats {
        &self.stats
    }
}

unsafe impl<Alloc: GlobalAlloc> GlobalAlloc for CountingAllocator<Alloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.stats.alloc_count.fetch_add(1, Ordering::Relaxed);
            self.stats
                .allocated
                .fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
    }
}
