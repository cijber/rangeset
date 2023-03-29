use crate::{Range, RangeSet};
use crate::internal::LinearRangeAdder;
use std::collections::Bound;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::ops::Bound::{Excluded, Included, Unbounded};

#[cfg(feature = "smallvec")]
impl<T: Ord + Debug, const N: usize> From<smallvec::SmallVec<[Range<T>; N]>> for RangeSet<T> {
    fn from(mut value: smallvec::SmallVec<[Range<T>; N]>) -> Self {
        value.sort_by(|l, r| l.start_pos().cmp(&r.start_pos()));
        let mut adder = LinearRangeAdder::with_capacity(value.len());
        for item in value {
            adder.add(item);
        }

        adder.finalize()
    }
}

impl<T: Ord + Debug> From<Vec<Range<T>>> for RangeSet<T> {
    fn from(mut value: Vec<Range<T>>) -> Self {
        value.sort_by(|l, r| l.start_pos().cmp(&r.start_pos()));
        let mut adder = LinearRangeAdder::with_capacity(value.len());
        for item in value {
            adder.add(item);
        }

        adder.finalize()
    }
}

impl<T: Ord + Debug, I: Into<Range<T>>, const N: usize> From<[I; N]> for RangeSet<T> {
    fn from(value: [I; N]) -> Self {
        let mut value = value.map(Into::into);
        value.sort_by(|l, r| l.start_pos().cmp(&r.start_pos()));
        let mut adder = LinearRangeAdder::with_capacity(N);
        for item in value {
            adder.add(item.into());
        }

        adder.finalize()
    }
}

impl<T: Ord + Debug> From<(Bound<T>, Bound<T>)> for Range<T> {
    fn from(value: (Bound<T>, Bound<T>)) -> Self {
        Range {
            start: value.0,
            end: value.1,
        }
    }
}

impl<T: Ord + Debug> From<RangeFull> for Range<T> {
    fn from(_: RangeFull) -> Self {
        Range::unbound()
    }
}

impl<T: Ord + Debug> From<RangeTo<T>> for Range<T> {
    fn from(range: RangeTo<T>) -> Self {
        Range::new(Unbounded, Excluded(range.end))
    }
}

impl<T: Ord + Debug> From<RangeToInclusive<T>> for Range<T> {
    fn from(range: RangeToInclusive<T>) -> Self {
        Range::new(Unbounded, Included(range.end))
    }
}

impl<T: Ord + Debug> From<RangeInclusive<T>> for Range<T> {
    fn from(range: RangeInclusive<T>) -> Self {
        let (start, end) = range.into_inner();
        Range::new(Included(start), Included(end))
    }
}

impl<T: Ord + Debug> From<std::ops::Range<T>> for Range<T> {
    fn from(value: std::ops::Range<T>) -> Self {
        Range::new(Included(value.start), Excluded(value.end))
    }
}

impl<T: Ord + Debug> From<RangeFrom<T>> for Range<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Range::new(Included(value.start), Unbounded)
    }
}