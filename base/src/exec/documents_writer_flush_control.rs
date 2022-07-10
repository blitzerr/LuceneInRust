/// Ref: DWFC. Controls flushing for the DWPT.
///
/// This is the supplier of the DWPT. When the `DocumentsWriter` needs to index documents, it asks
/// the DWFC for a page. The DWFC pulls out a page from the PerthreadPool and hands it over.
pub(crate) struct DocumentWriterFlushControl {}

impl DocumentWriterFlushControl {
    fn obtain_and_lock(&mut self) {}
}
