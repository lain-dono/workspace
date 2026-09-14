use super::table::Column;
use slotmap::{SecondaryMap, SlotMap};
use std::collections::HashSet;
use std::iter::{FilterMap, Map};

mod query;

pub use self::query::{Filter, Where};

type Values<'a> = slotmap::basic::Values<'a, Field, Meta>;
type ValuesMut<'a> = slotmap::basic::ValuesMut<'a, Field, Meta>;

slotmap::new_key_type! {
    pub struct Record;
    pub struct Field;
    pub struct Table;
}

#[derive(Default)]
pub struct Database {
    pub tables: SlotMap<Table, TableMeta>,
}

#[derive(Default)]
pub struct TableMeta {
    pub name: String,
    pub fields: SlotMap<Field, Meta>,
    pub records: SlotMap<Record, RecordMeta>,
}

impl TableMeta {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: SlotMap::default(),
            records: SlotMap::default(),
        }
    }

    pub fn add_field(&mut self, meta: Meta) -> Field {
        self.fields.insert(meta)
    }

    pub fn update(&mut self, record: Record, values: impl IntoIterator<Item = Value>) {
        if self.records.contains_key(record) {
            for (mut field, value) in self.record_mut(record).zip(values) {
                field.insert(value);
            }
        }
    }

    pub fn append(&mut self, values: impl IntoIterator<Item = Value>) {
        for (mut field, value) in self.empty().zip(values) {
            field.insert(value);
        }
    }

    pub fn append_default(&mut self) {
        for mut field in self.empty() {
            field.insert_default();
        }
    }

    pub fn remove(&mut self, record: Record) {
        if self.records.remove(record).is_some() {
            for mut meta in self.record_mut(record) {
                meta.remove();
            }
        }
    }

    pub fn values<'a>(
        &'a mut self,
        record: Record,
    ) -> FilterMap<Values<'a>, impl FnMut(&'a Meta) -> Option<&'a Value>> {
        self.fields
            .values()
            .filter_map(move |meta| meta.data.get(record))
    }

    pub fn values_mut<'a>(
        &'a mut self,
        record: Record,
    ) -> FilterMap<ValuesMut<'a>, impl FnMut(&'a mut Meta) -> Option<&'a mut Value>> {
        self.fields
            .values_mut()
            .filter_map(move |meta| meta.data.get_mut(record))
    }

    fn empty<'a>(&'a mut self) -> Map<ValuesMut<'a>, impl FnMut(&'a mut Meta) -> Mut<'a>> {
        let record = self.records.insert(RecordMeta {});
        self.record_mut(record)
    }

    fn record_mut<'a>(
        &'a mut self,
        record: Record,
    ) -> Map<ValuesMut<'a>, impl FnMut(&'a mut Meta) -> Mut<'a>> {
        self.fields
            .values_mut()
            .map(move |meta| Mut { record, meta })
    }
}

pub struct Mut<'a> {
    record: Record,
    meta: &'a mut Meta,
}

impl Mut<'_> {
    pub fn value(&self) -> Option<&Value> {
        self.meta.data.get(self.record)
    }

    pub fn value_mut(&mut self) -> Option<&mut Value> {
        self.meta.data.get_mut(self.record)
    }

    pub fn insert(&mut self, value: Value) {
        self.meta.data.insert(self.record, value);
    }

    pub fn insert_default(&mut self) {
        self.insert(self.meta.desc.default.clone());
    }

    pub fn remove(&mut self) {
        self.meta.data.remove(self.record);
    }
}

#[derive(Debug)]
pub struct Description {
    pub name: String,
    pub ty: Ty,
    pub default: Value,
}

#[derive(Debug)]
pub struct Meta {
    pub desc: Description,
    pub data: SecondaryMap<Record, Value>,

    pub column: Column,
}

impl Meta {
    pub fn singleline(name: impl Into<String>) -> Self {
        Self {
            desc: Description {
                name: name.into(),
                ty: Ty::Text { singleline: true },
                default: Value::Text(String::new()),
            },
            column: Column::auto().resizable(true),
            data: SecondaryMap::default(),
        }
    }

    pub fn variants(name: impl Into<String>, many: bool, variants: Vec<String>) -> Self {
        Self {
            desc: Description {
                name: name.into(),
                ty: Ty::Enum { variants, many },
                default: Value::Enum(HashSet::default()),
            },
            column: Column::auto().resizable(true),
            data: SecondaryMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct RecordMeta {}

#[derive(Debug)]
pub enum Ty {
    Blob,
    Text { singleline: bool },
    Link { table: Table, many: bool },
    Enum { variants: Vec<String>, many: bool },
}

#[derive(Clone, Debug)]
pub enum Value {
    Blob(Vec<u8>),
    Text(String),
    Link(HashSet<Record>),
    Enum(HashSet<String>),
}

impl Value {
    pub fn text(data: impl Into<String>) -> Self {
        Self::Text(data.into())
    }

    pub fn variants<T: Into<String>>(data: impl IntoIterator<Item = T>) -> Self {
        Self::Enum(data.into_iter().map(Into::into).collect())
    }
}
