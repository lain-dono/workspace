use bevy::{
    platform::collections::HashMap,
    ptr::{OwningPtr, Ptr, PtrMut},
};
use std::{borrow::Cow, marker::PhantomData, ptr::NonNull};

struct OwningBox<'w> {
    ptr: OwningPtr<'w>,
    drop: for<'a> unsafe fn(OwningPtr<'a>),
}

impl<'w> OwningBox<'w> {
    fn new<T>(value: T) -> Self {
        unsafe {
            assert_ne!(std::mem::size_of::<T>(), 0, "found ZST");

            let ptr = std::alloc::alloc(std::alloc::Layout::new::<T>()).cast::<T>();
            std::ptr::write(ptr, value);

            Self {
                ptr: OwningPtr::new(NonNull::new_unchecked(ptr.cast())),
                drop: Self::drop_ptr::<T> as _,
            }
        }
    }

    unsafe fn drop_ptr<T>(x: OwningPtr<'_>) {
        let ptr = x.as_ptr();

        // SAFETY: Contract is required to be upheld by the caller.
        unsafe { x.drop_as::<T>() }

        unsafe { std::alloc::dealloc(ptr, std::alloc::Layout::new::<T>()) }
    }

    fn drop(self) {
        unsafe { (self.drop)(self.ptr) }
    }
}

enum LocalItem<'w> {
    Unknown,
    Ref(Ptr<'w>),
    Mut(PtrMut<'w>),
    Own(OwningBox<'w>),
}

pub struct Local<'w> {
    values: Vec<LocalItem<'w>>,
    mapper: HashMap<Cow<'static, str>, usize>,
    marker: PhantomData<&'w ()>,
}

impl<'w> Local<'w> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            mapper: HashMap::new(),
            marker: PhantomData,
        }
    }

    pub fn set_or_insert<T: Sized + 'static>(&mut self, name: impl Into<String>, value: T) {
        let name = name.into();
        let new = OwningBox::new(value);

        if let Some(&index) = self.mapper.get(name.as_str()) {
            let old = std::mem::replace(&mut self.values[index], LocalItem::Own(new));
            assert!(matches!(old, LocalItem::Unknown));
        } else {
            let index = self.values.len();
            self.values.push(LocalItem::Own(new));
            self.mapper.insert(name.into(), index);
        }
    }

    pub fn drop(&mut self, name: impl AsRef<str>) {
        if let Some(&index) = self.mapper.get(name.as_ref()) {
            match std::mem::replace(&mut self.values[index], LocalItem::Unknown) {
                LocalItem::Own(inner) => inner.drop(),
                _ => unreachable!(),
            }
        }
    }

    pub fn get_ref(&self, name: impl AsRef<str>) -> Option<Ptr<'_>> {
        let index = self.mapper.get(name.as_ref()).copied()?;
        match &self.values[index] {
            LocalItem::Unknown => None,
            LocalItem::Ref(p) => Some(*p),
            LocalItem::Mut(p) => Some(p.as_ref()),
            LocalItem::Own(p) => Some(p.ptr.as_ref()),
        }
    }

    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<PtrMut<'_>> {
        let index = self.mapper.get(name.as_ref()).copied()?;
        match &mut self.values[index] {
            LocalItem::Unknown => None,
            LocalItem::Ref(_) => None,
            LocalItem::Mut(p) => Some(p.reborrow()),
            LocalItem::Own(p) => Some(p.ptr.as_mut()),
        }
    }

    /// # Safety
    /// no
    pub unsafe fn cast_ref<'p, T>(&self, name: &str) -> Option<&'p T> {
        Some(unsafe { &*self.get_ref(name)?.as_ptr().cast::<T>() })
    }

    /// # Safety
    /// no
    pub unsafe fn cast_mut<'p, T>(&mut self, name: &str) -> Option<&'p mut T> {
        Some(unsafe { &mut *self.get_ref(name)?.as_ptr().cast::<T>() })
    }

    pub fn add_ref<T>(&mut self, name: impl Into<Cow<'static, str>>, value: &'w T) {
        let index = self.values.len();
        self.values.push(LocalItem::Ref(Ptr::from(value)));
        self.mapper.insert(name.into(), index);
    }

    pub fn add_mut<T>(&mut self, name: impl Into<Cow<'static, str>>, value: &'w mut T) {
        let index = self.values.len();
        self.values.push(LocalItem::Mut(PtrMut::from(value)));
        self.mapper.insert(name.into(), index);
    }

    pub fn add_own<T: Sized + 'static>(&mut self, name: impl Into<String>, value: T) {
        let name = name.into();
        let new = OwningBox::new(value);

        let index = self.values.len();
        self.values.push(LocalItem::Own(new));
        self.mapper.insert(name.into(), index);
    }
}
