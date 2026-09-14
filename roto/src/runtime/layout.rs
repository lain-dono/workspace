/// Layout of a Roto or Rust type
///
/// The following must hold:
///  - align must be greater than 0
///  - align must be a power of 2
///  - size must be a multiple of align
///
///  Note that the last condition is not guaranteed by `std::alloc::Layout`, but
///  it will hold for a [`std::alloc::Layout`] created for any Rust type. This
///  is why there is no conversion from [`std::alloc::Layout`], but we do use it
///  internally to generate the layouts for Rust types.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    size: usize,
    align: usize,
}

impl Layout {
    /// Retrieve the layout for a given type
    pub const fn of<T>() -> Self {
        let layout = std::alloc::Layout::new::<T>();
        Self::from_size_align(layout.size(), layout.align())
    }

    /// Create a new [`Layout`]
    ///
    /// This function will panic if the conditions documented on [`Layout`] do not hold.
    pub const fn from_size_align(size: usize, align: usize) -> Self {
        assert!(align > 0);
        assert!(align.is_power_of_two());
        assert!(size.is_multiple_of(align));
        Self { size, align }
    }

    /// The size of the layout in bytes
    pub const fn size(&self) -> usize {
        self.size
    }

    /// The alignment of the layout in bytes
    pub const fn align(&self) -> usize {
        self.align
    }

    /// Create a [`Layout`] that could contain either of the operand [`Layout`]s
    ///
    /// This is mostly useful for enums.
    pub const fn union(self, other: Self) -> Self {
        // The conditions that align > 0 and that align is a power of 2 hold
        // trivially, but the condition that size is a multiple of align
        // doesn't hold necessarily, so we need to correct for it.
        //
        // Take for instance the union of:
        //  - `Layout { size: 6, align: 2 }`
        //  - `Layout { size: 4, align: 4 }`
        // Taking the max of both fields yields:
        //  - `Layout { size: 6, align: 4 }`
        // Where the size is not a multiple of align
        let align = if self.align > other.align {
            self.align
        } else {
            other.align
        };
        let size = if self.size > other.size {
            self.size
        } else {
            other.size
        };
        let size = size.next_multiple_of(align);
        Self { size, align }
    }

    pub fn concat(iter: impl IntoIterator<Item = Self>) -> Self {
        let init = LayoutBuilder::new();
        iter.into_iter().fold(init, LayoutBuilder::concat).finish()
    }

    pub const fn offset_by(self, n: usize) -> usize {
        LayoutBuilder::from_size_align(n, 1).add(self)
    }
}

/// Builder for [`Layout`]
///
/// This builds layouts in the C representation style.
pub struct LayoutBuilder {
    size: usize,
    align: usize,
}

impl Default for LayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutBuilder {
    /// Create a new [`LayoutBuilder`]
    pub const fn new() -> Self {
        Self { size: 0, align: 1 }
    }

    pub const fn from_size_align(size: usize, align: usize) -> Self {
        Self { size, align }
    }

    pub const fn of<T>() -> Self {
        let layout = std::alloc::Layout::new::<T>();
        Self::from_size_align(layout.size(), layout.align())
    }

    /// Add a [`Layout`] to this layout builder.
    ///
    /// The added layout will be properly aligned and the size and alignment of
    /// the builder will be updated accordingly.
    ///
    /// Returns the offset at which the layout was added
    pub const fn add(&mut self, layout: Layout) -> usize {
        let offset = self.size.next_multiple_of(layout.align());
        self.align = if layout.align() < self.align {
            self.align
        } else {
            layout.align()
        };
        self.size = offset + layout.size();
        offset
    }

    const fn concat(mut self, layout: Layout) -> Self {
        self.add(layout);
        self
    }

    /// Generate the final [`Layout`]
    pub const fn finish(self) -> Layout {
        let size = self.size.next_multiple_of(self.align);
        Layout::from_size_align(size, self.align)
    }
}
