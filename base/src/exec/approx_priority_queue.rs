use crate::error::internal::InternalErr;
use std::fmt::Debug;
use std::mem::take;
use tracing::trace;

/// In Lucene this is implemented in the class `ApproximatePriorityQueue.java`.
///
/// The implementation uses an ArrayList (Rust analog of vec) and a bit map underneath for
/// efficiency. The priority order is only cared for, for the first 64 elements. This is where
/// the bitset comes in handy. The priority order is determined by the weight which is usually
/// set to the number of bytes in RAM. For all other elements that are expected to be placed at a
/// spot greater than 64 are just appended to the array list. Lucene uses a bitwise operator to
/// get the number of leading zeros. The lesser the number of zeros, the higher the weight.
///
/// This implementation is kept simple. Here we use a separate Array for the top priority
/// elements and then we use a smallvec.
pub(crate) type WeightTy = u8;
const SIZE: usize = std::mem::size_of::<WeightTy>() * 8;

#[derive(Debug)]
pub(crate) struct ApproximatePriorityQueue<T> {
    pages: Vec<Option<T>>,
    /// A bitmap to track the taken slot for the first 64 slots. A set bit indicates a taken slot.
    used_slots: usize,
}

impl<T: PartialEq + Debug> ApproximatePriorityQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            // we are pre-allocating the vec-deq.
            pages: (0..SIZE).into_iter().map(|_| None).collect(),
            used_slots: 0,
        }
    }

    pub(crate) fn add(&mut self, entry: T, weight: WeightTy) -> eyre::Result<()> {
        trace!(?entry);
        // The first step is to determine the location of this new element. We use the number of
        // leading zeros as the proxy for the weight.
        let expected_slot = weight.leading_zeros();
        trace!(?expected_slot, ?entry, ?weight);

        // In case the expected slot is taken, the desirable slot is the first un-occupied slot
        // to the left. Remember, the highest priority element is the 0th element.
        if expected_slot < SIZE.try_into().unwrap() {
            let free_slots = !self.used_slots;
            let destination_slot = expected_slot + (free_slots >> expected_slot).trailing_zeros();
            trace!(?destination_slot);
            assert!(
                destination_slot >= expected_slot,
                "check failed: `{destination_slot} >= {expected_slot}`"
            );

            if destination_slot < SIZE.try_into().unwrap() {
                let destination_slot = destination_slot as usize;
                // If this is one of the pages with enough weights to be in the first 64 slots, set
                // the used_slot marker for it and then insert it in its appropriate slot.
                self.used_slots |= 1usize << destination_slot;
                let old = std::mem::replace(&mut self.pages[destination_slot], Some(entry));
                assert!(old.is_none(), "Expected None but found {old:?}");
            } else {
                self.pages.push(Some(entry));
            }
        } else {
            self.pages.push(Some(entry));
        }

        trace!(?self.pages, "After inserting");

        Ok(())
    }

    /// Returns an element that matches the provided predicate. Prefer the most weighty elements
    /// for a match first failing which start at the end and return the first matching element.
    pub(crate) fn poll<P>(&mut self, p: P) -> eyre::Result<Option<T>>
    where
        P: Fn(&T) -> bool,
    {
        let mut idx = 0;
        if let Some(y) = loop {
            let next_used_slot_post_idx = idx
                + self
                    .used_slots
                    .checked_shr(idx)
                    .ok_or(InternalErr::IllegalOperation {
                        op: format!("right shift by {idx}"),
                        on: self.used_slots,
                        err: "Shift too large for value type".to_string(),
                    })?
                    .trailing_zeros();
            let next_used_slot_post_idx_usz = next_used_slot_post_idx as usize;
            if next_used_slot_post_idx_usz >= SIZE.into() {
                break None;
            }

            if self
                .pages
                .get(next_used_slot_post_idx_usz)
                // This should not panic if the used slot bit-map is correct and if it is not
                // correct then we might as well just panic.
                .unwrap()
                .as_ref()
                .and_then(|x| p(x).then_some(x)) // validate the predicate
                .is_some()
            // This will be some only if the predicate matches in the last call.
            {
                self.used_slots &= !(1 << next_used_slot_post_idx);
                break take(&mut self.pages[next_used_slot_post_idx_usz]);
            } else {
                // The element in the next unused slot does not match the predicate, let's find
                // the next filled slot and repeat.
                idx = next_used_slot_post_idx + 1;
            }
        } {
            Ok(Some(y))
        } else {
            Ok(
                if let Some(matching_idx) = self
                    .pages
                    .iter()
                    .rev()
                    .enumerate()
                    .take(self.pages.len() - SIZE)
                    .find_map(|(i, x)| x.as_ref().and_then(|xx| p(xx).then_some(i)))
                {
                    let len = self.pages.len();
                    trace!(?matching_idx, ?len, "Polling");
                    take(&mut self.pages[len - matching_idx - 1])
                } else {
                    None
                },
            )
        }
    }

    /// removes the given element from the queue. The return value is true if the item was
    /// successfully detected.
    pub(crate) fn remove(&mut self, entry: T) -> bool {
        if let Some(i) = self.pages.iter().enumerate().find_map(|(i, item)| {
            if let Some(item) = item {
                if *item == entry {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            if i >= SIZE {
                self.pages.remove(i);
            } else {
                self.used_slots &= !(1 << i);
                take(&mut self.pages[i]);
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.used_slots == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::exec::approx_priority_queue::{ApproximatePriorityQueue, SIZE};
    use tracing::trace;
    //use tracing_test::traced_test;

    #[test]
    //#[traced_test]
    fn test_add() {
        let mut a = ApproximatePriorityQueue::new();
        a.add(2, 2).unwrap();
        assert!(!a.is_empty());
        a.add(1, 1).unwrap();
        a.add(0, 0).unwrap();

        // Adding 0 should increase the size of the vector by 1.
        assert_eq!(a.pages.len(), SIZE + 1);

        let mut v = vec![None; SIZE - 2];
        v.push(Some(2));
        v.push(Some(1));
        v.push(Some(0));
        assert_eq!(a.pages, v);

        assert_eq!(a.poll(|_| true).unwrap().unwrap(), 2);
        assert_eq!(a.poll(|_| true).unwrap().unwrap(), 1);
        assert_eq!(a.poll(|_| true).unwrap().unwrap(), 0);
        assert!(a.is_empty());
    }

    #[test]
    fn test_collision() {
        let mut a = ApproximatePriorityQueue::new();
        a.add(2, 2).unwrap();
        assert!(!a.is_empty());
        a.add(1, 1).unwrap();
        a.add(0, 0).unwrap();
        a.add(3, 3).unwrap(); // This collides with 2 and therefore gets to pushed.

        trace!(?a.pages);

        assert_eq!(a.pages.len(), SIZE + 2);
        let mut v = vec![None; SIZE - 2];
        v.push(Some(2));
        v.push(Some(1));
        v.push(Some(0));
        v.push(Some(3));
        assert_eq!(a.pages, v);
    }

    #[test]
    fn test_remove() {
        let mut a = ApproximatePriorityQueue::new();
        a.add(2, 2).unwrap();
        assert!(!a.is_empty());
        a.add(1, 1).unwrap();
        a.add(0, 0).unwrap();
        a.add(3, 3).unwrap();

        assert!(a.remove(2));
        assert!(!a.remove(2));
        assert!(a.remove(3));
        assert!(a.remove(0));
        assert!(a.remove(1));
        assert_eq!(a.pages.len(), SIZE);
    }

    #[test]
    fn test_predicate() {
        let mut a = ApproximatePriorityQueue::new();
        a.add(2, 2).unwrap();
        assert!(!a.is_empty());
        a.add(1, 1).unwrap();
        a.add(0, 0).unwrap();
        a.add(3, 3).unwrap();

        assert_eq!(a.poll(|x| x % 2 == 1).unwrap().unwrap(), 1);
        assert_eq!(a.poll(|x| x % 2 == 0).unwrap().unwrap(), 2);
        assert_eq!(a.poll(|x| x % 2 == 1).unwrap().unwrap(), 3);
        assert_eq!(a.poll(|x| x % 2 == 0).unwrap().unwrap(), 0);
    }
}
