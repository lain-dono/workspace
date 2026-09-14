use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

#[cfg_attr(
    feature = "codec",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "T: serde::Serialize")),
    serde(bound(deserialize = "T: serde::Deserialize<'de>"))
)]
pub struct Handle<T> {
    #[cfg_attr(feature = "codec", serde(serialize_with = "rc_serde::serialize"))]
    #[cfg_attr(feature = "codec", serde(deserialize_with = "rc_serde::deserialize"))]
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    #[must_use]
    pub fn into_inner(self) -> Rc<RefCell<T>> {
        self.inner
    }

    #[must_use]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

/// Custom Serde serializers for `Rc<RefCell<...>>`
#[cfg(feature = "codec")]
pub mod rc_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Serializer for `Rc<RefCell<T>`.
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize<S, T>(rc: &Rc<RefCell<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*rc.borrow(), serializer)
    }

    /// Serializer for `Option<Rc<RefCell<T>>`.
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize_option<S, T>(
        val: &Option<Rc<RefCell<T>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match val {
            Some(rc) => serialize(rc, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Serializer for `Vec<Rc<RefCell<T>>`.
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize_vec<S, T>(val: &Vec<Rc<RefCell<T>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let mut seq = serializer.serialize_seq(Some(val.len()))?;
        for item in val {
            seq.serialize_element(&*item.borrow())?;
        }
        seq.end()
    }

    /// Deserializer for `Rc<RefCell<T>`
    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Rc<RefCell<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Rc::new(RefCell::new(value)))
    }

    /// Deserializer for `Option<Rc<RefCell<T>>`
    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize_option<'de, D, T>(
        deserializer: D,
    ) -> Result<Option<Rc<RefCell<T>>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;
        Ok(opt.map(|value| Rc::new(RefCell::new(value))))
    }

    /// Deserializer for `Vec<Rc<RefCell<T>>`
    #[allow(clippy::missing_errors_doc)]
    // grcov-excl-start
    pub fn deserialize_vec<'de, D, T>(deserializer: D) -> Result<Vec<Rc<RefCell<T>>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Ok(vec
            .into_iter()
            .map(|item| Rc::new(RefCell::new(item)))
            .collect())
    } // grcov-excl-end
}
