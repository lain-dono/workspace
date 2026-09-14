use super::{align_down, align_up};
use core::{alloc::Layout, ptr};

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Debug)]
pub struct Allocator {
    current: usize,
    start: usize,
}

impl Allocator {
    /// Creates a new bump allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    #[allow(dead_code)]
    pub const fn new(start: usize, end: usize) -> Self {
        Self {
            current: end,
            start,
        }
    }
}

impl Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let ptr = self.current.checked_sub(layout.size());
        ptr.map(|ptr| align_down(ptr, layout.align()))
            .filter(|&ptr| ptr >= self.start)
            .map_or(ptr::null_mut(), |ptr| {
                self.current = ptr;
                ptr as *mut u8
            })
    }
}
