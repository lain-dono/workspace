/*
impl From<ast::StructTypes> for StructTypes {
    fn from(value: ast::StructTypes) -> Self {
        Self {
            name: value.name,
            attributes: value
                .attributes
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let name = value.attr_name.clone();
                    let mut value = StructAttributeType::from(value);
                    value.attr_index = u32::try_from(index).unwrap_or_default();
                    (name.into(), value)
                })
                .collect(),
            methods: HashMap::new(),
        }
    }
}
    */
