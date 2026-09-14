use super::{Key, Occupied, Slot, SlotUnion, Slots, Vacant};
use serde::{
    de::{self, Deserialize, Deserializer, Error, MapAccess, SeqAccess},
    ser::{Serialize, SerializeStruct, Serializer},
};
use std::{marker::PhantomData, mem::ManuallyDrop};

struct SerdeSlot<T> {
    value: Option<T>,
    version: u32,
}

impl<T: Serialize> Serialize for SerdeSlot<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut slot = serializer.serialize_struct("SerdeSlot", 2)?;
        slot.serialize_field("value", &self.value)?;
        slot.serialize_field("version", &self.version)?;
        slot.end()
    }
}

#[automatically_derived]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for SerdeSlot<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            Value,
            Version,
        }

        const FIELDS: &[&str] = &["value", "version"];

        struct FieldVisitor;

        impl de::Visitor<'_> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.write_str("field identifier")
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
                match value {
                    0 => Ok(Field::Value),
                    1 => Ok(Field::Version),
                    _ => Err(E::unknown_field(&value.to_string(), FIELDS)),
                }
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "value" => Ok(Field::Value),
                    "version" => Ok(Field::Version),
                    _ => Err(E::unknown_field(value, FIELDS)),
                }
            }

            fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                match value {
                    b"value" => Ok(Field::Value),
                    b"version" => Ok(Field::Version),
                    _ => Err(E::unknown_field(&format!("{value:?}"), FIELDS)),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct Visitor<'de, T: Deserialize<'de>> {
            marker: PhantomData<SerdeSlot<T>>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visitor<'de, T> {
            type Value = SerdeSlot<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct SerdeSlot")
            }

            #[inline]
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let value = seq.next_element::<Option<T>>()?;
                let value = value.ok_or_else(|| Error::invalid_length(0, &self))?;
                let version = seq.next_element::<u32>()?;
                let version = version.ok_or_else(|| Error::invalid_length(1, &self))?;
                Ok(SerdeSlot { value, version })
            }

            #[inline]
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut value = None;
                let mut version = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Value if value.is_some() => {
                            return Err(A::Error::duplicate_field("value"));
                        }
                        Field::Version if version.is_some() => {
                            return Err(A::Error::duplicate_field("version"));
                        }

                        Field::Value => value = Some(map.next_value::<Option<T>>()?),
                        Field::Version => version = Some(map.next_value::<u32>()?),
                    }
                }

                let value = value.ok_or_else(|| Error::missing_field("value"))?;
                let version = version.ok_or_else(|| Error::missing_field("version"))?;
                Ok(SerdeSlot { value, version })
            }
        }

        let visitor = Visitor {
            marker: PhantomData::<SerdeSlot<T>>,
            lifetime: PhantomData,
        };

        deserializer.deserialize_struct("SerdeSlot", FIELDS, visitor)
    }
}

impl<T: Serialize> Serialize for Slot<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let version = self.version;
        let value = match self.get() {
            Occupied(value) => Some(value),
            Vacant(_) => None,
        };
        SerdeSlot { version, value }.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Slot<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let serde_slot: SerdeSlot<T> = Deserialize::deserialize(deserializer)?;
        let occupied = serde_slot.version % 2 == 1;
        if occupied ^ serde_slot.value.is_some() {
            Err(de::Error::custom("inconsistent occupation in Slot"))
        } else {
            Ok(Self {
                u: match serde_slot.value {
                    Some(value) => SlotUnion {
                        value: ManuallyDrop::new(value),
                    },
                    None => SlotUnion { next_free: 0 },
                },
                version: serde_slot.version,
            })
        }
    }
}

impl<K: Key, V: Serialize> Serialize for Slots<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.slots.serialize(serializer)
    }
}

impl<'de, K: Key, V: Deserialize<'de>> Deserialize<'de> for Slots<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut slots: Vec<Slot<V>> = Deserialize::deserialize(deserializer)?;

        if slots.len() >= u32::MAX as usize {
            return Err(de::Error::custom("too many slots"));
        }

        // Ensure the first slot exists and is empty for the sentinel.
        if slots.first().is_none_or(|slot| slot.version % 2 == 1) {
            return Err(de::Error::custom("first slot not empty"));
        }

        slots[0].version = 0;
        slots[0].u.next_free = 0;

        // We have our slots, rebuild freelist.
        let mut num_elems = 0;
        let mut next_free = slots.len();
        for (i, slot) in slots[1..].iter_mut().enumerate() {
            if slot.occupied() {
                num_elems += 1;
            } else {
                slot.u.next_free = next_free as u32;
                next_free = i + 1;
            }
        }

        Ok(Self {
            num_elems,
            slots,
            free_head: next_free as u32,
            key: PhantomData,
        })
    }
}
