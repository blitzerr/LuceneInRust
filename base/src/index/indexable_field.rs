#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum IndexableField {
    IntPoint {
        name: String,
        value: i32,
        props: Vec<FieldProperties>,
    },
    //IntRange(CommonParams),
    //Text(CommonParams),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum FieldProperties {
    Tokenized,
}

impl IndexableField {
    pub(crate) fn new_int(name: &str, value: i32) -> Self {
        IndexableField::IntPoint {
            name: name.to_owned(),
            value,
            props: vec![],
        }
    }
    pub(crate) fn new_int_with_props(name: &str, value: i32, props: Vec<FieldProperties>) -> Self {
        IndexableField::IntPoint {
            name: name.to_owned(),
            value,
            props,
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            IndexableField::IntPoint { name, .. } => name,
            //IndexableField::IntRange(p) => &p.name,
            //IndexableField::Text(p) => &p.name,
        }
    }
}
