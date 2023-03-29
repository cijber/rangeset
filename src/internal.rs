use std::fmt::Debug;
use crate::{Bound, Range, RangeSet, RangeVec};

#[derive(Debug)]
pub struct LinearRangeAdder<T: Ord + Debug> {
    items: RangeVec<Range<T>>,
    last: Option<Range<T>>,
}

impl<T: Ord + Debug> Default for LinearRangeAdder<T> {
    #[inline]
    fn default() -> Self {
        Self::with_capacity(4)
    }
}

impl<T: Ord + Debug> LinearRangeAdder<T> {
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
        if self.last.as_ref().map_or(false, |x| x.end == Bound::Unbounded) {
            return true;
        }

        match self.last.take() {
            None => self.last = Some(range),
            Some(mut v) => {
                debug_assert!(v.start_pos() <= range.start_pos(), "range ({:?}) added to adder is lower than previous range {:?}", range, v);
                if v.end_pos() < range.start_pos() {
                    self.items.push(v);
                    self.last = Some(range);
                } else {
                    if range.end_pos() > v.end_pos() {
                        v.end = range.end;
                    }

                    self.last = Some(v);
                }
            }
        }

        self.last.as_ref().map_or(false, |x| x.end == Bound::Unbounded)
    }

    pub fn finalize(mut self) -> RangeSet<T> {
        if let Some(v) = self.last {
            self.items.push(v);
        }

        RangeSet { items: self.items }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::LinearRangeAdder;
    use crate::{r, RangeSet};

    #[test]
    pub fn adder() {
        let mut adder = LinearRangeAdder::new();
        adder.add(r!(..20));
        adder.add(r!(..4));
        adder.add(r!(10..));
        adder.add(r!(50..));
        let fin = adder.finalize();

        assert_eq!(RangeSet::unbound(), fin);

        let mut adder = LinearRangeAdder::new();
        adder.add(r!(..1));
        adder.add(r!(4 >..));
        let fin = adder.finalize();

        assert_eq!(fin.items, [r!(..1), r!(4 >..)].into());
    }
}