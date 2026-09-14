use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;

/// The actual data stored in a [`Key`].
///
/// This implements [`Ord`](std::cmp::Ord) so keys can be stored in e.g.
/// [`BTreeMap`](std::collections::BTreeMap), but the order of keys is
/// unspecified.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct KeyData {
    pub(crate) index: u32,
    pub(crate) version: NonZeroU32,
}

impl KeyData {
    pub(crate) const fn new(index: u32, version: u32) -> Self {
        debug_assert!(version > 0);

        let version = unsafe { NonZeroU32::new_unchecked(version | 1) };
        Self { index, version }
    }

    pub(crate) const fn raw(index: u32, version: NonZeroU32) -> Self {
        Self { index, version }
    }

    pub(crate) const fn null() -> Self {
        Self::raw(u32::MAX, unsafe { NonZeroU32::new_unchecked(1) })
    }

    /// Returns the key data as a 64-bit integer. No guarantees about its value
    /// are made other than that passing it to [`from_ffi`](Self::from_ffi)
    /// will return a key equal to the original.
    ///
    /// With this you can easily pass slot map keys as opaque handles to foreign
    /// code. After you get them back you can confidently use them in your slot
    /// map without worrying about unsafe behavior as you would with passing and
    /// receiving back references or pointers.
    ///
    /// This is not a substitute for proper serialization, use [`serde`] for
    /// that. If you are not doing FFI, you almost surely do not need this
    /// function.
    ///
    /// [`serde`]: crate#serialization-through-serde-no_std-support-and-unstable-features
    pub const fn as_ffi(self) -> u64 {
        ((self.version.get() as u64) << 32) | self.index as u64
    }

    /// Iff `value` is a value received from `k.as_ffi()`, returns a key equal
    /// to `k`. Otherwise the behavior is safe but unspecified.
    pub const fn from_ffi(value: u64) -> Self {
        Self::raw((value & 0xffff_ffff) as u32, unsafe {
            // Ensure version is odd.
            NonZeroU32::new_unchecked(((value >> 32) | 1) as u32)
        })
    }
}

impl fmt::Debug for KeyData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.version.get())
    }
}

impl Default for KeyData {
    fn default() -> Self {
        Self::null()
    }
}

impl Hash for KeyData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // A derived Hash impl would call write_u32 twice. We call write_u64
        // once, which is beneficial if the hasher implements write_u64
        // explicitly.
        state.write_u64(self.as_ffi())
    }
}

/// Key used to access stored values in a slot map.
///
/// Do not use a key from one slot map in another. The behavior is safe but
/// non-sensical (and might panic in case of out-of-bounds).
///
/// To prevent this, it is suggested to have a unique key type for each slot
/// map. You can create new key types using [`new_key_type!`], which makes a
/// new type identical to [`DefaultKey`], just with a different name.
///
/// This trait is intended to be a thin wrapper around [`KeyData`], and all
/// methods must behave exactly as if we're operating on a [`KeyData`] directly.
/// The internal unsafe code relies on this, therefore this trait is `unsafe` to
/// implement. It is strongly suggested to simply use [`new_key_type!`] instead
/// of implementing this trait yourself.
pub trait Key:
    Into<KeyData> + From<KeyData> + Copy + Clone + Default + Ord + std::hash::Hash + std::fmt::Debug
{
}

impl<T> Key for T where
    T: Into<KeyData>
        + From<KeyData>
        + Copy
        + Clone
        + Default
        + Ord
        + std::hash::Hash
        + std::fmt::Debug
{
}
