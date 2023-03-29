use crate::{BoundExt, Range, RangeSet};
use crate::internal::LinearRangeAdder;
use std::collections::Bound;

#[cfg(feature = "smallvec")]
impl<T: Ord, const N: usize> From<smallvec::SmallVec<[Range<T>; N]>> for RangeSet<T> {
    fn from(mut value: smallvec::SmallVec<[Range<T>; N]>) -> Self {
        value.sort_by(|a, b| a.from.cmp_lower_bound(&b.from));
        let mut adder = LinearRangeAdder::with_capacity(value.len());
        for item in value {
            adder.add(item);
        }

        adder.finalize()
    }
}

impl<T: Ord> From<Vec<Range<T>>> for RangeSet<T> {
    fn from(mut value: Vec<Range<T>>) -> Self {
        value.sort_by(|a, b| a.from.cmp_lower_bound(&b.from));
        let mut adder = LinearRangeAdder::with_capacity(value.len());
        for item in value {
            adder.add(item);
        }

        adder.finalize()
    }
}

impl<T: Ord, I: Into<Range<T>>, const N: usize> From<[I; N]> for RangeSet<T> {
    fn from(mut value: [I; N]) -> Self {
        let mut value = value.map(Into::into);
        value.sort_by(|a, b| a.from.cmp_lower_bound(&b.from));
        let mut adder = LinearRangeAdder::with_capacity(N);
        for item in value {
            adder.add(item.into());
        }

        adder.finalize()
    }
}

impl<T: Ord> From<(Bound<T>, Bound<T>)> for Range<T> {
    fn from(value: (Bound<T>, Bound<T>)) -> Self {
        Range {
            from: value.0,
            to: value.1,
        }
    }
}
