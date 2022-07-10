use std::collections::HashSet;

use crate::index::indexable_field::IndexableField;

#[derive(Debug)]
pub struct Doc {
    fields: HashSet<IndexableField>,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            fields: HashSet::new(),
        }
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<IndexableField> {
        self.fields.iter()
    }

    pub fn add(&mut self, field: IndexableField) {
        self.fields.insert(field);
    }

    pub fn remove(&mut self, name: &str) {
        self.fields.retain(|field| field.name() != name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove() {
        let mut d = Doc::new();
        d.add(IndexableField::new_int("price", 23));
        d.add(IndexableField::new_int("units", 21));

        let mut it = d.iter();

        assert!(HashSet::from(["price", "units"]).contains(it.next().unwrap().name()));
        assert!(HashSet::from(["price", "units"]).contains(it.next().unwrap().name()));

        d.remove("price");

        let mut it = d.iter();
        assert_eq!(it.next().unwrap().name(), "units");
        assert_eq!(it.next(), None);
    }
}
