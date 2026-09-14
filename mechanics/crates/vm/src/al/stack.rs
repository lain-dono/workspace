// #[inline]
// fn align_down(addr: usize, align: usize) -> usize {
//     debug_assert!(align.is_power_of_two());
//     addr & !(align - 1)
// }

// #[inline]
// fn align_up(addr: usize, align: usize) -> usize {
//     debug_assert!(align.is_power_of_two());
//     (addr + align - 1) & !(align - 1)
// }
//#![feature(ptr_internals)]

use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc},
    any::TypeId,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
};

// #[repr(transparent)]
// pub struct Exclusive<T: ?Sized> {
//     inner: T,
// }

// unsafe impl<T: ?Sized> Sync for Exclusive<T> {}

#[repr(transparent)]
struct Unique<T: ?Sized>(NonNull<T>, PhantomData<T>);

unsafe impl<T: Send + ?Sized> Send for Unique<T> {}
unsafe impl<T: Sync + ?Sized> Sync for Unique<T> {}

impl<T: ?Sized> Copy for Unique<T> {}
impl<T: ?Sized> Clone for Unique<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Sized> Unique<T> {
    #[inline]
    pub const fn dangling() -> Self {
        // SAFETY: mem::align_of() returns a valid, non-null pointer. The
        // conditions to call new_unchecked() are thus respected.
        unsafe { Unique::new_unchecked(std::ptr::dangling_mut::<T>()) }
    }
}

impl<T: ?Sized> Unique<T> {
    #[inline]
    const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: the caller must guarantee that `ptr` is non-null.
        Self(unsafe { NonNull::new_unchecked(ptr) }, PhantomData)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub const fn cast<U>(self) -> Unique<U> {
        // SAFETY: Unique::new_unchecked() creates a new unique and needs
        // the given pointer to not be null.
        // Since we are passing self as a pointer, it cannot be null.
        unsafe { Unique::new_unchecked(self.as_ptr().cast()) }
    }
}

pub fn main() {
    #[repr(C, align(256))]
    struct Dropable(u32);
    impl Drop for Dropable {
        fn drop(&mut self) {
            println!(
                "{} is dropped, ptr: {:?}",
                self.0,
                std::ptr::from_ref::<Self>(self)
            );
        }
    }

    struct DropPanic;
    impl Drop for DropPanic {
        fn drop(&mut self) {
            panic!();
        }
    }

    let mut slab = Alloc::new();

    println!("1:");
    slab.push(1usize);
    slab.push(1i32);
    slab.push(Dropable(1));
    slab.push(String::from("1"));

    println!("2:");
    slab.push(2usize);
    slab.push(2i32);
    slab.push(Dropable(2));
    slab.push(String::from("2"));

    slab.push(DropPanic);

    println!("3:");
    slab.push(3usize);
    slab.push(3i32);
    slab.push(Dropable(3));
    slab.push(String::from("3"));

    print!("iter::<usize>: ");
    for v in slab.iter::<usize>() {
        print!("{v}, ");
    }

    print!("\niter::<i32>: ");
    for v in slab.iter::<i32>() {
        print!("{v}, ");
    }

    print!("\niter::<String>: ");
    for v in slab.iter::<String>() {
        print!("{v:#?}, ");
    }

    println!("\nend");

    let slab = std::sync::Mutex::new(slab);
    let _ = std::panic::catch_unwind(|| {
        slab.lock().unwrap().clear();
    });

    let _slab = slab
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    println!("ok");
}

pub trait Message: 'static + Sized + Send + Sync {}

impl<T> Message for T where T: 'static + Sized + Send + Sync {}

#[repr(C, align(8))]
struct Tag {
    type_id: TypeId,
    type_align: usize,
    it_and_past_size: usize,
    drop_fn: Option<fn(*mut u8)>,
}

impl Tag {
    unsafe fn drop_data(self, ptr: *mut u8) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(ptr);
        }
    }
}

struct RawAlloc {
    ptr: Unique<Tag>,
    cap: usize,
    padding: usize,
    max_align: usize,
}

impl RawAlloc {
    fn new() -> Self {
        Self {
            ptr: Unique::<Tag>::dangling().cast(),
            cap: 0,
            padding: 0,
            max_align: mem::align_of::<Tag>(),
        }
    }

    unsafe fn grow(&mut self, end: usize) {
        let size = mem::size_of::<Tag>();
        let align = mem::align_of::<Tag>();
        debug_assert!(align.is_power_of_two());
        debug_assert_ne!(size, 0);

        let old_offset = self.offset();
        let (layout, ptr, cap) = if self.cap == 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(2 * size, align);
                let ptr = alloc(layout);
                (layout, ptr, 2 * size)
            }
        } else {
            let cap = (2 * self.cap - self.padding).saturating_add(self.max_align - align);
            assert!(cap < isize::MAX as usize);
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.cap, align);
                let ptr = realloc(self.ptr.cast().as_ptr(), layout, cap);
                (layout, ptr, cap)
            }
        };
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        self.ptr = unsafe { Unique::new_unchecked(ptr).cast() };

        let padding = self.next_padding(old_offset);
        unsafe { ptr::copy(ptr.add(self.padding), ptr.add(padding), end) };

        self.cap = cap;
        self.padding = padding;
    }

    fn offset(&self) -> usize {
        self.as_ptr().align_offset(self.max_align)
    }

    fn next_padding(&self, old_offset: usize) -> usize {
        let new_offset = self.offset();
        let padding = self.max_align + self.padding + new_offset - old_offset;
        padding % self.max_align
    }

    fn as_ptr(&self) -> *mut u8 {
        let ptr: *mut u8 = self.ptr.cast().as_ptr();
        unsafe { ptr.add(self.padding) }
    }

    fn capacity(&self) -> usize {
        self.cap - self.padding
    }

    fn acknowledge_align_of<M: Message>(&mut self) {
        self.max_align = self.max_align.max(mem::align_of::<M>());
    }
}

impl Drop for RawAlloc {
    fn drop(&mut self) {
        if self.cap != 0 {
            let align = mem::align_of::<Tag>();
            let layout = unsafe { Layout::from_size_align_unchecked(self.cap, align) };
            unsafe { dealloc(self.ptr.cast().as_ptr(), layout) };
        }
    }
}

pub struct Alloc {
    buf: RawAlloc,
    len: usize,
    end: usize,
    last_size: usize,
}

impl Default for Alloc {
    fn default() -> Self {
        Self {
            buf: RawAlloc::new(),
            len: 0,
            end: 0,
            last_size: 0,
        }
    }
}

impl Alloc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        while self.pop() {}
    }

    pub fn push<M: Message>(&mut self, message: M) {
        let inbounds_safe = mem::size_of::<Tag>()
            + mem::align_of::<M>()
            + mem::size_of::<M>()
            + mem::align_of::<Tag>();

        while self.end + inbounds_safe >= self.buf.capacity() {
            unsafe { self.buf.grow(self.end) };
        }

        self.buf.acknowledge_align_of::<M>();
        let msg = self.next_msg::<M>(self.end);
        let end = self.next_cursor::<M>(msg);
        let it_size = end - self.end;
        let it_and_past_size = it_size + self.last_size;
        let tag = Tag {
            type_id: TypeId::of::<M>(),
            type_align: mem::align_of::<M>(),
            it_and_past_size,
            drop_fn: if mem::needs_drop::<M>() {
                Some(|p| unsafe { ptr::drop_in_place(p.cast::<M>()) })
            } else {
                None
            },
        };

        unsafe {
            let start = self.buf.as_ptr();
            start.add(self.end).cast::<Tag>().write(tag);
            start.add(msg).cast::<M>().write(message);
        }

        self.len += 1;
        self.end = end;
        self.last_size = it_size;
    }

    fn pop(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }
        self.len -= 1;
        self.end -= self.last_size;
        unsafe {
            let start: *mut u8 = self.buf.as_ptr();
            let tag = start.add(self.end).cast::<Tag>().read();
            self.last_size = tag.it_and_past_size - self.last_size;
            let msg = self.next_untyped(self.end, mem::size_of::<Tag>(), tag.type_align);
            let ptr = start.add(msg);
            tag.drop_data(ptr);
        }
        true
    }

    pub fn iter<M: Message>(&self) -> Iter<'_, M> {
        Iter {
            inner: self,
            cursor: 0,
            past_size: 0,
            _marker: PhantomData,
        }
    }

    fn next_msg<M: Message>(&self, cursor: usize) -> usize {
        self.next_untyped(cursor, mem::size_of::<Tag>(), mem::align_of::<M>())
    }

    fn next_cursor<M: Message>(&self, next_msg: usize) -> usize {
        self.next_untyped(next_msg, mem::size_of::<M>(), mem::align_of::<Tag>())
    }

    fn next_untyped(&self, offset: usize, size: usize, align: usize) -> usize {
        let next = offset + size;
        next + unsafe { self.buf.as_ptr().add(next) }.align_offset(align)
    }
}

impl Drop for Alloc {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, M: Message> {
    inner: &'a Alloc,
    cursor: usize,
    past_size: usize,
    _marker: PhantomData<&'a M>,
}

impl<'a, M: Message> Iterator for Iter<'a, M> {
    type Item = &'a M;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            inner,
            cursor,
            past_size,
            ..
        } = self;

        while *cursor != inner.end {
            unsafe {
                let start: *mut u8 = inner.buf.as_ptr();
                let tag = start.add(*cursor).cast::<Tag>().as_ref().unwrap();
                let msg = inner.next_untyped(*cursor, mem::size_of::<Tag>(), tag.type_align);
                let it_size = tag.it_and_past_size - *past_size;
                *cursor += it_size;
                *past_size = it_size;
                if TypeId::of::<M>() != tag.type_id {
                    continue;
                }
                return start.add(msg).cast::<M>().as_ref();
            }
        }
        None
    }
}
