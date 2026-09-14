use bevy_ptr::{Ptr, PtrMut};
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::NonNull;

pub struct Memory {
    memory: NonNull<u8>,
    layout: Layout,
}

impl Drop for Memory {
    fn drop(&mut self) {
        if self.layout != Layout::new::<()>() {
            unsafe { dealloc(self.memory.as_ptr(), self.layout) }
        }
    }
}

impl Memory {
    #[inline]
    pub unsafe fn get<T>(&self, offset: usize) -> NonNull<T> {
        debug_assert_eq!(offset & (align_of::<T>() - 1), 0);
        unsafe { self.memory.add(offset).cast::<T>() }
    }

    pub unsafe fn write<T>(&mut self, offset: usize, val: T) {
        unsafe { self.get::<T>(offset).write(val) }
    }
}

#[derive(Debug)]
pub struct MemoryBuilder {
    memory: NonNull<u8>,
    layout: Layout,
    cursor: usize,
}

unsafe impl Send for MemoryBuilder {}
unsafe impl Sync for MemoryBuilder {}

impl Clone for MemoryBuilder {
    fn clone(&self) -> Self {
        let Some(memory) = NonNull::new(unsafe { alloc(self.layout) }) else {
            handle_alloc_error(self.layout);
        };

        Self {
            memory,
            layout: self.layout,
            cursor: self.cursor,
        }
    }
}

impl Drop for MemoryBuilder {
    fn drop(&mut self) {
        if self.layout != Layout::new::<()>() {
            unsafe { dealloc(self.memory.as_ptr(), self.layout) }
        }
    }
}

impl Default for MemoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBuilder {
    pub const fn new() -> Self {
        Self {
            memory: NonNull::dangling(),
            layout: Layout::new::<()>(),
            cursor: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == 0
    }

    pub fn len(&self) -> usize {
        self.cursor
    }

    pub fn build(self) -> Memory {
        let layout = unsafe { Layout::from_size_align_unchecked(self.cursor, self.layout.align()) };
        let layout = layout.pad_to_align();

        if let Some(memory) = NonNull::new(unsafe { alloc(layout) }) {
            unsafe { memory.copy_from_nonoverlapping(self.memory, layout.size()) }
            Memory { memory, layout }
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    pub unsafe fn get<T>(&self, offset: usize) -> NonNull<T> {
        debug_assert_eq!(offset & (align_of::<T>() - 1), 0);
        unsafe { self.memory.add(offset).cast::<T>() }
    }

    pub unsafe fn write<T>(&mut self, offset: usize, val: T) {
        debug_assert_eq!(offset & (align_of::<T>() - 1), 0);
        unsafe { self.memory.add(offset).cast::<T>().write(val) }
    }

    #[inline]
    pub unsafe fn ptr<'a>(&self, offset: usize) -> Ptr<'a> {
        unsafe { Ptr::new(self.memory.add(offset)) }
    }

    #[inline]
    pub unsafe fn ptr_mut<'a>(&self, offset: usize) -> PtrMut<'a> {
        unsafe { PtrMut::new(self.memory.add(offset)) }
    }

    pub unsafe fn push_map<T>(&mut self, val: impl FnOnce(usize) -> T) -> usize {
        unsafe {
            let offset = self.alloc(Layout::new::<T>());
            self.write(offset, val(offset));
            offset
        }
    }

    pub unsafe fn push<T>(&mut self, val: T) -> usize {
        unsafe {
            let offset = self.alloc(Layout::new::<T>());
            self.write(offset, val);
            offset
        }
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> usize {
        let start = align_up(self.cursor, layout.align());
        let cursor = start + layout.size();

        let align = layout.align().max(self.layout.align());
        let size = align_up(cursor, align).next_power_of_two();

        let layout = Layout::from_size_align(size, align).unwrap();
        if layout.align() > self.layout.align() || layout.size() > self.layout.size() {
            if let Some(memory) = NonNull::new(unsafe { alloc(layout) }) {
                if self.layout != Layout::new::<()>() {
                    // SAFETY: the previously allocated block cannot overlap the newly allocated block.
                    // The safety contract for `dealloc` must be upheld by the caller.
                    unsafe {
                        let count = self.layout.size();
                        memory.copy_from_nonoverlapping(self.memory, count);
                        dealloc(self.memory.as_ptr(), self.layout);
                    }
                }
                self.memory = memory;
                self.layout = layout;
            } else {
                handle_alloc_error(layout)
            }
        }

        self.cursor = cursor;

        start
    }
}

#[inline(always)]
pub const fn align_up(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

#[test]
fn test_align_up() {
    assert_eq!(align_up(0, 4), 0);
    assert_eq!(align_up(1, 4), 4);
    assert_eq!(align_up(2, 4), 4);
    assert_eq!(align_up(3, 4), 4);
    assert_eq!(align_up(4, 4), 4);
    assert_eq!(align_up(5, 4), 8);
}
