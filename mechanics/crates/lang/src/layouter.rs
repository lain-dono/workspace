use crate::arena::{Arena, Handle, HandleVec};
use crate::ty;
use core::{fmt::Display, num::NonZeroU32, ops};

/// A newtype struct where its only valid values are powers of 2

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Alignment(NonZeroU32);

impl Alignment {
    pub const ONE: Self = Self(NonZeroU32::new(1).unwrap());
    pub const TWO: Self = Self(NonZeroU32::new(2).unwrap());
    pub const FOUR: Self = Self(NonZeroU32::new(4).unwrap());
    pub const EIGHT: Self = Self(NonZeroU32::new(8).unwrap());
    pub const SIXTEEN: Self = Self(NonZeroU32::new(16).unwrap());

    pub const fn new(n: u32) -> Option<Self> {
        if n.is_power_of_two() {
            // SAFETY: value can't be 0 since we just checked if it's a power of 2
            Some(Self(unsafe { NonZeroU32::new_unchecked(n) }))
        } else {
            None
        }
    }

    /// # Panics
    /// If `width` is not a power of 2
    pub fn from_width(width: u8) -> Self {
        Self::new(width as u32).unwrap()
    }

    /// Returns whether or not `n` is a multiple of this alignment.
    pub const fn is_aligned(&self, n: u32) -> bool {
        // equivalent to: `n % self.0.get() == 0` but much faster
        n & (self.0.get() - 1) == 0
    }

    /// Round `n` up to the nearest alignment boundary.
    pub const fn round_up(&self, n: u32) -> u32 {
        // equivalent to:
        // match n % self.0.get() {
        //     0 => n,
        //     rem => n + (self.0.get() - rem),
        // }

        let mask = self.0.get() - 1;
        (n + mask) & !mask
    }
}

impl Display for Alignment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.get().fmt(f)
    }
}

impl ops::Mul<u32> for Alignment {
    type Output = u32;

    fn mul(self, rhs: u32) -> Self::Output {
        self.0.get() * rhs
    }
}

impl ops::Mul for Alignment {
    type Output = Alignment;

    fn mul(self, rhs: Alignment) -> Self::Output {
        // SAFETY: both lhs and rhs are powers of 2, the result will be a power of 2
        Self(unsafe { NonZeroU32::new_unchecked(self.0.get() * rhs.0.get()) })
    }
}

/// Size and alignment information for a type.

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeLayout {
    pub size: u32,
    pub align: Alignment,
}

impl TypeLayout {
    /// Produce the stride as if this type is a base of an array.
    pub const fn to_stride(&self) -> u32 {
        self.align.round_up(self.size)
    }
}

/// Helper processor that derives the sizes of all types.
///
/// `Layouter` uses the default layout algorithm/table, described in
/// [WGSL §4.3.7, "Memory Layout"]
///
/// A `Layouter` may be indexed by `Handle<Type>` values: `layouter[handle]` is the
/// layout of the type whose handle is `handle`.
///
/// [WGSL §4.3.7, "Memory Layout"](https://gpuweb.github.io/gpuweb/wgsl/#memory-layouts)
#[derive(Debug, Default)]
pub struct Layouter {
    /// Layouts for types in an arena.
    layouts: HandleVec<ty::Type, TypeLayout>,
}

impl ops::Index<Handle<ty::Type>> for Layouter {
    type Output = TypeLayout;

    fn index(&self, handle: Handle<ty::Type>) -> &TypeLayout {
        &self.layouts[handle]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum LayoutErrorInner {
    #[error("Array element type {0:?} doesn't exist")]
    InvalidArrayElementType(Handle<ty::Type>),

    #[error("Struct member[{0}] type {1:?} doesn't exist")]
    InvalidStructMemberType(u32, Handle<ty::Type>),

    #[error("Type width must be a power of two")]
    NonPowerOfTwoWidth,
}

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[error("Error laying out type {ty:?}: {inner}")]
pub struct LayoutError {
    pub ty: Handle<ty::Type>,
    pub inner: LayoutErrorInner,
}

impl LayoutErrorInner {
    const fn with(self, ty: Handle<ty::Type>) -> LayoutError {
        LayoutError { ty, inner: self }
    }
}

impl Layouter {
    /// Remove all entries from this `Layouter`, retaining storage.
    pub fn clear(&mut self) {
        self.layouts.clear();
    }

    /// Extend this `Layouter` with layouts for any new entries in `gctx.types`.
    ///
    /// Ensure that every type in `gctx.types` has a corresponding [`TypeLayout`]
    /// in [`self.layouts`].
    ///
    /// Some front ends need to be able to compute layouts for existing types
    /// while module construction is still in progress and new types are still
    /// being added. This function assumes that the `TypeLayout` values already
    /// present in `self.layouts` cover their corresponding entries in `types`,
    /// and extends `self.layouts` as needed to cover the rest. Thus, a front
    /// end can call this function at any time, passing its current type and
    /// constant arenas, and then assume that layouts are available for all
    /// types.
    #[allow(clippy::or_fun_call)]
    pub fn update(&mut self, types: &Arena<ty::Type>) -> Result<(), LayoutError> {
        for (ty_handle, ty) in types.iter().skip(self.layouts.len()) {
            let size = ty.inner.size();

            let layout = match ty.inner {
                ty::TypeInner::Scalar(scalar) => {
                    let err = LayoutErrorInner::NonPowerOfTwoWidth.with(ty_handle);
                    let align = Alignment::new(scalar.width().unwrap_or(1)).ok_or(err)?;
                    TypeLayout { size, align }
                }

                ty::TypeInner::Pointer { .. } => TypeLayout {
                    size,
                    align: Alignment::ONE,
                },

                ty::TypeInner::Array(base, _size, _stride) => TypeLayout {
                    size,
                    align: if base < ty_handle {
                        self[base].align
                    } else {
                        return Err(LayoutErrorInner::InvalidArrayElementType(base).with(ty_handle));
                    },
                },

                ty::TypeInner::Struct(ref members, size) => {
                    let mut align = Alignment::ONE;

                    for (index, &ty::StructMember(_, member_ty, _)) in members.iter().enumerate() {
                        align = if member_ty < ty_handle {
                            align.max(self[member_ty].align)
                        } else {
                            let index = index as u32;
                            return Err(LayoutErrorInner::InvalidStructMemberType(
                                index, member_ty,
                            )
                            .with(ty_handle));
                        };
                    }

                    TypeLayout { size, align }
                }
            };

            debug_assert!(size <= layout.size);

            self.layouts.insert(ty_handle, layout);
        }

        Ok(())
    }
}
