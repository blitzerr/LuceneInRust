use crate::document::{doc::Doc, terms::Term};

use super::documents_writer::DocumentsWriter;

pub struct IndexWriter {
    documents_writer: DocumentsWriter,
}

impl IndexWriter {
    pub fn add_documents<It>(&mut self, docs_iter: It)
    where
        It: IntoIterator<Item = Doc>,
    {
        self.update_documents(None, docs_iter)
    }
    pub fn update_documents<It>(&mut self, _del_term: Option<Term>, _docs_iter: It)
    where
        It: IntoIterator<Item = Doc>,
    {
        todo!()
    }
}
