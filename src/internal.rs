use std::fmt::Debug;
use crate::{Bound, BoundChecks, Range, RangeSet, RangeVec};

#[derive(Debug)]
pub struct LinearRangeAdder<T: Ord> {
    items: RangeVec<Range<T>>,
    last: Option<Range<T>>,
}

impl<T: Ord> Default for LinearRangeAdder<T> {
    #[inline]
    fn default() -> Self {
        Self::with_capacity(4)
    }
}

impl<T: Ord> LinearRangeAdder<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        LinearRangeAdder {
            items: RangeVec::with_capacity(cap),
            last: None,
        }
    }

    pub fn add(&mut self, range: Range<T>) -> bool {
        match self.last.take() {
            None => self.last = Some(range),
            Some(mut v) => {
                debug_assert!(v.from.is_above_lower_bound(&range.from), "range added to adder is lower than previous range");
                if v.to.is_above_upper_bound(&range.from) {
                    self.items.push(v);
                    self.last = Some(range);
                } else {
                    if range.to.is_below_upper_bound(&v.to) {
                        v.to = range.to;
                    }

                    self.last = Some(v);
                }
            }
        }

        self.last.as_ref().map_or(false, |x| x.to == Bound::Unbounded)
    }

    pub fn finalize(mut self) -> RangeSet<T> {
        if let Some(v) = self.last {
            self.items.push(v);
        }

        RangeSet { items: self.items }
    }
}