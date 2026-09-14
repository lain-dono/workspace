use super::{Field, Meta, Record, RecordMeta, TableMeta, Value};

#[derive(Clone, Copy)]
pub struct ViewRef<'a> {
    record: Record,
    storage: &'a slotmap::SlotMap<Field, Meta>,
}

impl<'a> ViewRef<'a> {
    #[inline]
    pub fn get(self, field: Field) -> Option<&'a Value> {
        self.meta(field)?.data.get(self.record)
    }

    pub fn filter(self, filter: &Filter) -> Option<Self> {
        filter.is(self).then_some(self)
    }

    pub fn fields<I: IntoIterator<Item = Field> + Copy>(
        self,
        fields: I,
    ) -> std::iter::Map<
        <I as std::iter::IntoIterator>::IntoIter,
        impl FnMut(Field) -> Option<&'a Value>,
    > {
        fields.into_iter().map(move |field| self.get(field))
    }

    #[inline]
    pub fn meta(self, field: Field) -> Option<&'a Meta> {
        self.storage.get(field)
    }

    #[inline]
    pub fn is_some_and(self, field: Field, f: impl FnOnce(&Value) -> bool) -> bool {
        self.get(field).is_some_and(f)
    }

    #[inline]
    pub fn is_none_or(self, field: Field, f: impl FnOnce(&Value) -> bool) -> bool {
        self.get(field).is_none_or(f)
    }

    #[inline]
    pub fn is_some(self, field: Field) -> bool {
        self.meta(field)
            .is_some_and(|meta| meta.data.contains_key(self.record))
    }

    #[inline]
    pub fn is_none(self, field: Field) -> bool {
        self.meta(field)
            .is_none_or(|meta| !meta.data.contains_key(self.record))
    }

    #[inline]
    pub fn includes(self, field: Field, value: &Value) -> bool {
        self.is_some_and(field, |record| match (record, value) {
            (Value::Text(record), Value::Text(value)) => record.contains(value),
            _ => false,
        })
    }

    #[inline]
    pub fn excludes(self, field: Field, value: &Value) -> bool {
        self.is_none_or(field, |record| match (record, value) {
            (Value::Text(record), Value::Text(value)) => !record.contains(value),
            _ => true,
        })
    }

    #[inline]
    pub fn is_disjoint(self, field: Field, value: &Value) -> bool {
        self.is_none_or(field, |record| match (record, value) {
            (Value::Enum(record), Value::Enum(value)) => record.is_disjoint(value),
            (Value::Link(record), Value::Link(value)) => record.is_disjoint(value),
            _ => true, // TODO: maybe false?
        })
    }

    #[inline]
    pub fn is_subset(self, field: Field, value: &Value) -> bool {
        self.is_some_and(field, |record| match (record, value) {
            (Value::Enum(record), Value::Enum(value)) => record.is_subset(value),
            (Value::Link(record), Value::Link(value)) => record.is_subset(value),
            _ => false,
        })
    }

    #[inline]
    pub fn is_superset(self, field: Field, value: &Value) -> bool {
        self.is_some_and(field, |record| match (record, value) {
            (Value::Enum(record), Value::Enum(value)) => record.is_superset(value),
            (Value::Link(record), Value::Link(value)) => record.is_superset(value),
            _ => false,
        })
    }
}

impl TableMeta {
    #[inline]
    pub fn get(&self, record: Record, field: Field) -> Option<&Value> {
        self.fields.get(field)?.data.get(record)
    }

    #[inline]
    pub fn get_mut(&mut self, record: Record, field: Field) -> Option<&mut Value> {
        self.fields.get_mut(field)?.data.get_mut(record)
    }

    #[inline]
    pub fn query<'a>(
        &'a self,
        filter: &Filter,
    ) -> std::iter::FilterMap<
        slotmap::basic::Keys<'a, Record, RecordMeta>,
        impl FnMut(Record) -> Option<ViewRef<'a>>,
    > {
        let (records, storage) = (self.records.keys(), &self.fields);
        records.filter_map(move |record| ViewRef { record, storage }.filter(filter))
    }

    #[inline]
    pub fn query_values<I: IntoIterator<Item = Field> + Copy>(
        &self,
        filter: &Filter,
        fields: I,
    ) -> impl Iterator<Item = impl Iterator<Item = Option<&Value>>> {
        self.query(filter).map(move |view| view.fields(fields))
    }

    /// # Safety
    ///
    /// fields must be unique
    #[inline]
    pub unsafe fn query_mut<I: IntoIterator<Item = Field> + Copy>(
        &mut self,
        filter: &Filter,
        fields: I,
    ) -> impl Iterator<Item = impl Iterator<Item = Option<&'_ mut Value>>> {
        let storage = &mut self.fields;
        self.records.keys().filter_map(move |record| {
            if filter.is(ViewRef { record, storage }) {
                let storage: *mut slotmap::SlotMap<Field, Meta> = storage as *mut _;
                Some(fields.into_iter().map(move |field| {
                    unsafe { &mut *storage }
                        .get_mut(field)?
                        .data
                        .get_mut(record)
                }))
            } else {
                None
            }
        })
    }
}

#[derive(Clone)]
pub enum Filter {
    All(Vec<Where>),
    Any(Vec<Where>),
}

impl Filter {
    pub fn all(items: impl IntoIterator<Item = Where>) -> Self {
        Self::All(items.into_iter().collect())
    }

    pub fn any(items: impl IntoIterator<Item = Where>) -> Self {
        Self::Any(items.into_iter().collect())
    }

    fn is(&self, view: ViewRef<'_>) -> bool {
        match self {
            Self::All(items) => items.iter().all(|item| item.is(view)),
            Self::Any(items) => items.iter().any(|item| item.is(view)),
        }
    }
}

#[derive(Clone)]
pub enum Where {
    Filter(Filter),

    IsSome(Field),
    IsNone(Field),

    Includes(Field, Value),
    Excludes(Field, Value),

    IsDisjoint(Field, Value),
    IsSubset(Field, Value),
    IsSuperset(Field, Value),

    Compare(Field, Compare, Value),
}

impl Where {
    fn is(&self, view: ViewRef) -> bool {
        match *self {
            Self::Filter(ref filter) => filter.is(view),

            Self::IsSome(field) => view.is_some(field),
            Self::IsNone(field) => view.is_none(field),

            Self::Includes(field, ref value) => view.includes(field, value),
            Self::Excludes(field, ref value) => view.excludes(field, value),

            Self::IsDisjoint(field, ref value) => view.is_disjoint(field, value),
            Self::IsSubset(field, ref value) => view.is_subset(field, value),
            Self::IsSuperset(field, ref value) => view.is_superset(field, value),

            Self::Compare(field, compare, ref value) => compare.is(view, field, value),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Compare {
    Eq, // ==
    Ne, // !=

    Ge, // <=
    Gt, // <
    Le, // >=
    Lt, // >
}

impl Compare {
    fn is(self, view: ViewRef<'_>, field: Field, value: &Value) -> bool {
        let record = view.get(field);
        match self {
            Self::Eq => match (record, value) {
                (Some(Value::Blob(record)), Value::Blob(value)) => record == value,
                (Some(Value::Text(record)), Value::Text(value)) => record == value,
                _ => false,
            },
            Self::Ne => match (record, value) {
                (Some(Value::Blob(record)), Value::Blob(value)) => record != value,
                (Some(Value::Text(record)), Value::Text(value)) => record != value,
                _ => true,
            },

            Self::Ge => todo!(),
            Self::Gt => todo!(),
            Self::Le => todo!(),
            Self::Lt => todo!(),
        }
    }
}
