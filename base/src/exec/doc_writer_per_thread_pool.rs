use std::sync::atomic::{AtomicU64, Ordering};

struct DocumentsWriterPerThreadPool {
    pages: AtomicU64,
}

impl DocumentsWriterPerThreadPool {
    pub const fn new() -> Self {
        Self {
            pages: AtomicU64::new(0),
        }
    }
    /// Sets the bit and returns
    /// true: If the bit was un-set previously and is set after this operation.
    /// false: otherwise.
    pub fn set_bit(&self, i: u8) -> bool {
        let bit_i = 1u64 << (i as u64);
        (self.pages.fetch_or(bit_i, Ordering::AcqRel) & bit_i) == 0
    }
    /// Unset the bit and returns
    /// true: If the bit was set previously and now it is unset.
    /// false: otherwise.
    pub fn unset_bit(&self, i: u8) -> bool {
        let bit_i = 1u64 << (i as u64);
        (self.pages.fetch_and(!bit_i, Ordering::Release) & bit_i) != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::exec::doc_writer_per_thread_pool::DocumentsWriterPerThreadPool;

    #[test]
    fn test_bool() {
        let bool = DocumentsWriterPerThreadPool::new();

        assert_eq!(bool.set_bit(5), true);
        assert_eq!(bool.set_bit(5), false);
        assert_eq!(bool.set_bit(6), true);

        assert_eq!(bool.unset_bit(5), true);
        assert_eq!(bool.set_bit(5), true);

        assert_eq!(bool.set_bit(63), true);
    }
}
