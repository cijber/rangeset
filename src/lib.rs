use std::fmt::Debug;
use std::{mem};
use std::cmp::Ordering;
use std::ops::{Bound, RangeBounds, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use crate::Bound::{Excluded, Included, Unbounded};
use crate::internal::LinearRangeAdder;


mod internal;
mod conversions;

use crate::conversions::*;

#[cfg(feature = "smallvec")]
pub(crate) type RangeVec<T> = smallvec::SmallVec<[T; 5]>;

#[cfg(not(feature = "smallvec"))]
pub(crate) type RangeVec<T> = Vec<T>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RangeSet<T: Ord> {
    pub(crate) items: RangeVec<Range<T>>,
}

impl<T: Ord + Debug> Default for RangeSet<T> {
    fn default() -> Self {
        Self::with_capacity(4)
    }
}

impl<T: Ord + Debug> RangeSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Self::with_capacity(0)
    }

    pub fn unbound() -> Self {
        let mut items = RangeVec::with_capacity(1);
        items.push(Range::unbound());

        Self {
            items,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.items.len() == 1 && self.items[0].is_unbound()
    }

    pub fn with_capacity(data: usize) -> Self {
        RangeSet {
            items: RangeVec::with_capacity(data),
        }
    }

    #[inline]
    pub fn items(&self) -> impl Iterator<Item=&Range<T>> {
        self.items.iter()
    }

    pub fn contains(&self, other: &T) -> bool {
        for range in self.items() {
            if range.from.is_above_lower_bound(other) {
                if range.to.is_below_upper_bound(other) {
                    return true;
                } else {
                    // Window got overshot
                    break;
                }
            }
        }

        false
    }

    pub fn add(&mut self, range: Range<T>) {
        // If it's unbound then adding won't result into any change
        if self.is_unbound() {
            return;
        }

        // If given range is unbound, the whole set will become unbound
        if range.is_unbound() {
            self.items.clear();
            self.items.push(range);
            return;
        }

        // If it's empty, there's no need to recalculate
        if self.is_empty() {
            self.items.push(range);
            return;
        }

        let mut adder = LinearRangeAdder::new();
        let iter = self.items.drain(..);

        let mut range = Some(range);

        for item in iter {
            let item: Range<T> = item;

            if range.as_ref().map_or(false, |range| range.from.is_above_lower_bound(&item.from)) {
                if let Some(r) = range.take() {
                    if adder.add(r) {
                        break;
                    }
                }
            }

            if adder.add(item) {
                break;
            }
        }

        if let Some(r) = range {
            adder.add(r);
        }

        self.items = adder.finalize().items;
    }
}

impl<T: Ord + Clone + Debug> RangeSet<T> {
    pub fn union(&self, other: &Self) -> Self {
        if other.is_empty() {
            return self.clone();
        }

        if self.is_empty() {
            return other.clone();
        }

        if self.is_unbound() || other.is_unbound() {}

        let mut left_iter = self.items();
        let mut right_iter = other.items();

        let mut left = left_iter.next();
        let mut right = right_iter.next();

        let mut adder = LinearRangeAdder::new();

        loop {
            match (left, right) {
                (None, None) => break,
                (Some(l), Some(r)) => {
                    if r.from.is_above_lower_bound(&l.from) {
                        println!("R L={:?} / R={:?}", l, r);
                        if adder.add(r.clone()) {
                            break;
                        }

                        right = right_iter.next();
                    } else {
                        println!("L L={:?} / R={:?}", l, r);
                        if adder.add(l.clone()) {
                            break;
                        }

                        left = left_iter.next();
                    }
                }
                (Some(l), None) => {
                    if adder.add(l.clone()) {
                        break;
                    }

                    left = left_iter.next();
                }

                (None, Some(r)) => {
                    if adder.add(r.clone()) {
                        break;
                    }

                    right = right_iter.next();
                }
            }
        }

        adder.finalize()
    }

    pub fn invert(&self) -> RangeSet<T> {
        if self.is_empty() {
            return RangeSet::unbound();
        }

        if self.is_unbound() {
            return RangeSet::empty();
        }

        let mut items = RangeVec::with_capacity(self.items.len() + 2);
        let mut current = Range::unbound();

        for item in self.items() {
            if item.from == Unbounded {
                current.from = item.to.clone().invert();
                continue;
            }

            current.to = item.from.clone().invert();
            let last = mem::replace(&mut current, Range::new(item.to.clone().invert(), Unbounded));
            items.push(last);

            if item.to == Unbounded {
                return RangeSet { items };
            }
        }

        items.push(current);
        RangeSet { items }
    }

    pub fn intersection(&self) -> RangeSet<T> {
        let left = self.invert();
        let right = self.invert();

        left.union(&right).invert()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Range<T: Ord> {
    from: Bound<T>,
    to: Bound<T>,
}

impl<T: Ord> From<(T, T)> for Range<T> {
    fn from(value: (T, T)) -> Self {
        Range {
            from: Included(value.0),
            to: Excluded(value.1),
        }
    }
}

impl<T: Ord> Range<T> {
    pub fn new(from: Bound<T>, to: Bound<T>) -> Range<T> {
        Self {
            from,
            to,
        }
    }

    pub fn unbound() -> Range<T> {
        Self {
            from: Unbounded,
            to: Unbounded,
        }
    }

    #[inline]
    pub fn from(&self) -> Bound<&T> {
        self.from.as_ref()
    }

    #[inline]
    pub fn to(&self) -> Bound<&T> {
        self.to.as_ref()
    }

    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.from == Unbounded && self.to == Unbounded
    }

    #[inline]
    pub fn contains(&self, item: &T) -> bool {
        self.from.is_above_lower_bound(item) && self.to.is_below_upper_bound(item)
    }
}

impl<T: Ord + Clone> Range<T> {
    fn from_range<R: RangeBounds<T>>(value: R) -> Self {
        Range {
            from: value.start_bound().cloned(),
            to: value.end_bound().cloned(),
        }
    }
}

// #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
// pub enum Bound<T: Ord> {
//     Unbounded,
//     Excluded(T),
//     Included(T),
// }

pub trait BoundExt<T: Ord> {
    fn invert(self) -> Bound<T>;
    fn as_ref(&self) -> Bound<&T>;

    fn cmp_lower_bound(&self, rhs: &Bound<T>) -> Ordering;
    fn cmp_upper_bound(&self, rhs: &Bound<T>) -> Ordering;
}

impl<T: Ord> BoundExt<T> for Bound<T> {
    fn invert(self) -> Bound<T> {
        match self {
            Unbounded => Unbounded,
            Excluded(v) => Included(v),
            Included(v) => Excluded(v),
        }
    }

    #[inline]
    fn as_ref(&self) -> Bound<&T> {
        match &self {
            Unbounded => Unbounded,
            Excluded(t) => Excluded(t),
            Included(t) => Included(t),
        }
    }

    fn cmp_lower_bound(&self, rhs: &Bound<T>) -> Ordering {
        match (self.as_ref(), rhs.as_ref()) {
            (l, r) if l == r => Ordering::Equal,
            (_, Unbounded) => Ordering::Greater,
            (Unbounded, _) => Ordering::Less,
            (Excluded(l), Included(r)) if l < r => Ordering::Less,
            (Included(l), Excluded(r)) if l <= r => Ordering::Less,
            _ => Ordering::Greater,
        }
    }

    fn cmp_upper_bound(&self, rhs: &Bound<T>) -> Ordering {
        todo!()
    }
}

pub trait BoundChecks<Rhs = Self> {
    #[inline]
    fn is_below_lower_bound(&self, other: &Rhs) -> bool {
        !self.is_above_lower_bound(other)
    }

    fn is_above_lower_bound(&self, other: &Rhs) -> bool;
    fn is_below_upper_bound(&self, other: &Rhs) -> bool;

    #[inline]
    fn is_above_upper_bound(&self, other: &Rhs) -> bool {
        !self.is_below_upper_bound(other)
    }
}


impl<T: Ord> BoundChecks for Bound<T> {
    #[inline]
    fn is_below_lower_bound(&self, other: &Bound<T>) -> bool {
        return !self.is_above_lower_bound(other);
    }

    #[inline]
    fn is_above_lower_bound(&self, other: &Bound<T>) -> bool {
        match (self, other.as_ref()) {
            (Unbounded, _) => true,
            (_, Unbounded) => false,
            (Bound::Excluded(l), Bound::Included(r)) => {
                return l < r;
            }
            (Bound::Included(l), Bound::Included(r)) | (Bound::Included(l), Bound::Excluded(r)) | (Bound::Excluded(l), Bound::Excluded(r)) => {
                return l <= r;
            }
        }
    }

    #[inline]
    fn is_below_upper_bound(&self, other: &Bound<T>) -> bool {
        match (self, other.as_ref()) {
            (Unbounded, _) => true,
            (_, Unbounded) => false,
            (Bound::Excluded(l), Bound::Included(r)) => {
                return l > r;
            }
            (Bound::Included(l), Bound::Included(r)) | (Bound::Included(l), Bound::Excluded(r)) | (Bound::Excluded(l), Bound::Excluded(r)) => {
                return l >= r;
            }
        }
    }

    #[inline]
    fn is_above_upper_bound(&self, other: &Bound<T>) -> bool {
        return !self.is_below_upper_bound(other);
    }
}


impl<T: Ord> BoundChecks<T> for Bound<T> {
    #[inline]
    fn is_below_lower_bound(&self, other: &T) -> bool {
        return !self.is_above_lower_bound(other);
    }

    #[inline]
    fn is_above_lower_bound(&self, other: &T) -> bool {
        match self {
            Unbounded => true,
            Bound::Excluded(t) => t < other,
            Bound::Included(t) => t <= other,
        }
    }

    #[inline]
    fn is_below_upper_bound(&self, other: &T) -> bool {
        match self {
            Unbounded => true,
            Bound::Excluded(t) => t > other,
            Bound::Included(t) => t >= other,
        }
    }

    #[inline]
    fn is_above_upper_bound(&self, other: &T) -> bool {
        return !self.is_below_upper_bound(other);
    }
}

#[cfg(test)]
mod tests {
    use crate::Bound::{Excluded, Included};
    use super::*;

    #[test]
    fn union() {
        let mut left = RangeSet::new();
        let mut right = RangeSet::new();

        assert_eq!(RangeSet::empty(), left);
        let new = left.union(&right);
        assert_eq!(left, new);
        assert_eq!(right, new);

        left.add(Range::new(Unbounded, Bound::Included(4)));
        let new = left.union(&right);

        assert_eq!(left, new);
        let new_2 = right.union(&left);
        assert_eq!(new, new_2);

        right.add(Range::new(Bound::Included(4), Unbounded));
        let new = left.union(&right);
        assert_eq!(new, RangeSet::unbound());
        let new_2 = right.union(&left);
        assert_eq!(new, new_2);

        let mut left = RangeSet::new();
        let mut right = RangeSet::new();

        left.add(Range::new(Bound::Included(1), Bound::Included(3)));
        left.add(Range::new(Bound::Included(4), Bound::Included(6)));
        println!("{:?}", left);

        right.add(Range::new(Bound::Included(2), Bound::Included(5)));

        let new = left.union(&right);
        let new_2 = right.union(&left);
        assert_eq!(new, new_2);

        let mut expected = RangeSet::new();
        expected.add(Range::new(Bound::Included(1), Bound::Included(6)));
        assert_eq!(expected, new);
    }

    #[test]
    fn invert() {
        let test: RangeSet<usize> = RangeSet::unbound();
        assert_eq!(test.invert(), RangeSet::empty());
        assert_eq!(test.invert().invert(), test);

        let mut test = RangeSet::empty();
        test.add(Range::new(Bound::Unbounded, Excluded(4)));

        let mut expected = RangeSet::empty();
        expected.add(Range::new(Included(4), Unbounded));

        assert!(!test.contains(&4));
        assert!(test.invert().contains(&4));
        assert_eq!(test.invert(), expected);
    }

    #[test]
    fn intersection() {
        let left: RangeSet<usize> = RangeSet::unbound();
        let right: RangeSet<usize> = RangeSet::from([(Unbounded, Unbounded)]);

        let intersection = left.invert().union(&right.invert()).invert();
        println!("{:?}", intersection);
    }

    #[test]
    fn difference() {
        let left: RangeSet<usize> = RangeSet::unbound();
        let right: RangeSet<usize> = RangeSet::empty();
    }
}
