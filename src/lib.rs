//! A simple library with some boilerplate code to work with Range's and RangeSet's
//!
//! Currently every set operation depends on a mix of [`RangeSet::union`](RangeSet::union) and
//! [`RangeSet::invert`](RangeSet::invert) as those are the only 2 operations needed to implement
//! all other operations.
//!
//! This also reduces surface for errors, as only 2 functions have to be proven correct.
//!
//! This will change in the future

use std::fmt::Debug;
use std::{mem};
use std::cmp::Ordering;
use std::ops::{Deref, RangeBounds};
use crate::Bound::{Excluded, Included, Unbounded};
use crate::internal::LinearRangeAdder;


mod internal;
mod conversions;
mod macros;

/// Re-export for ease
pub use std::ops::Bound;

pub use crate::r as range;

/// The list type used for storing multiple ranges in a set
///
/// Disable the `smallvec` feature to use the std [Vec](Vec)
#[cfg(feature = "smallvec")]
pub type RangeVec<T> = smallvec::SmallVec<[T; 5]>;

/// The list type used for storing multiple ranges in a set
///
/// Enable the `smallvec` feature to use the smallvec's [`SmallVec`](smallvec::SmallVec)
#[cfg(not(feature = "smallvec"))]
pub type RangeVec<T> = Vec<T>;

/// A set of ranges
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RangeSet<T: Ord> {
    pub(crate) items: RangeVec<Range<T>>,
}

impl<T: Ord + Debug> Default for RangeSet<T> {
    fn default() -> Self {
        Self::with_capacity(5)
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

    /// If this is an empty set
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// If this set is unbounded or infinite
    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.items.len() == 1 && self.items[0].is_unbound()
    }

    /// Create a new set with given capacity
    #[inline]
    pub fn with_capacity(data: usize) -> Self {
        RangeSet {
            items: RangeVec::with_capacity(data),
        }
    }

    /// Returns an iterator with all ranges inside of this set
    #[inline]
    pub fn items(&self) -> impl Iterator<Item=&Range<T>> {
        self.items.iter()
    }

    /// Check if `other` falls within the ranges defined in this set
    pub fn contains(&self, other: &T) -> bool {
        for range in self.items() {
            if range.start_pos() < other {
                if range.end_pos() > other {
                    return true;
                }
            } else {
                // Window got overshot
                break;
            }
        }

        false
    }

    /// Add a new range to this set
    ///
    /// # Example
    ///
    /// ```rust
    /// use eater_rangeset::{r, range_set};
    ///
    /// let mut r = range_set![usize:];
    /// r.add(r!(4..));
    /// r.add(r!(3..5));
    ///
    /// assert_eq!(range_set![r!(3..)], r);
    /// ```
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

            if range.as_ref().map_or(false, |range| range.start_pos() < item.start_pos()) {
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
    /// Create an union of this set and given set
    ///
    /// # Example
    ///
    /// ```rust
    /// use eater_rangeset::{r, range_set};
    ///
    /// let left = range_set![r!(4>..)];
    /// let right = range_set![r!(0..7)];
    ///
    /// assert_eq!(range_set![r!(0..)], left.union(&right));
    /// ```
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
                    if r.start_pos() < l.start_pos() {
                        if adder.add(r.clone()) {
                            break;
                        }

                        right = right_iter.next();
                    } else {
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

    /// Invert current set, e.g. the result will match nothing this set matches
    ///
    /// # Example
    ///
    /// ```rust
    /// use eater_rangeset::{r, range_set};
    ///
    /// let t = range_set![r!(4>..)];
    ///
    /// assert_eq!(range_set![r!(..=4)], t.invert());
    /// ```
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
            if item.start == Unbounded {
                current.start = item.end.clone().invert();
                continue;
            }

            current.end = item.start.clone().invert();
            let last = mem::replace(&mut current, Range::new(item.end.clone().invert(), Unbounded));
            items.push(last);

            if item.end == Unbounded {
                return RangeSet { items };
            }
        }

        items.push(current);
        RangeSet { items }
    }

    /// Get the intersection of the 2 sets, or in other words, the places where the sets overlap
    ///
    /// # Example
    ///
    /// ```rust
    /// use eater_rangeset::{r, range_set};
    ///
    /// let left = range_set![r!(4..10), r!(20..30)];
    /// let right = range_set![r!(..5), r!(25..34)];
    ///
    /// assert_eq!(range_set![r!(4..5), r!(25..30)], left.intersection(&right));
    /// ```
    pub fn intersection(&self, rhs: &RangeSet<T>) -> RangeSet<T> {
        let left = self.invert();
        let right = rhs.invert();
        let inter = left.union(&right);

        inter.invert()
    }

    /// Get the difference of this set with given set, alike `lhs - rhs`
    ///
    /// # Example
    ///
    /// ```rust
    /// use eater_rangeset::{r, range_set};
    ///
    /// let left = range_set![r!(15..34)];
    /// let right = range_set![r!(3..20)];
    ///
    /// assert_eq!(range_set![r!(20..34)], left.difference(&right));
    /// // This method is asymmetric
    /// assert_eq!(range_set![r!(3..15)], right.difference(&left));
    /// ```
    pub fn difference(&self, rhs: &RangeSet<T>) -> RangeSet<T> {
        let left = self.invert();
        let mid = left.union(rhs);
        let res = mid.invert();
        res
    }

    /// Returns `true` if this set does not overlap in anyway with given set
    pub fn is_disjoint(&self, rhs: &RangeSet<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return true;
        }

        if self.is_unbound() || rhs.is_unbound() {
            return false;
        }

        let mut left_iter = self.items();
        let mut right_iter = rhs.items();

        let mut left = left_iter.next();
        let mut right = right_iter.next();

        loop {
            match (left, right) {
                (Some(l), Some(r)) => {
                    if r.start_pos() == l.start_pos() {
                        return false;
                    }

                    if r.start_pos() < l.start_pos() {
                        if r.end_pos() >= l.end_pos() {
                            return false;
                        } else {
                            right = right_iter.next();
                        }
                    } else {
                        if r.start_pos() < l.end_pos() {
                            return false;
                        } else {
                            left = left_iter.next();
                        }
                    }
                }

                _ => break,
            }
        }

        true
    }

    /// Returns `true` if this set overlaps anywhere with given set
    pub fn is_overlapping(&self, rhs: &RangeSet<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        if self.is_unbound() || rhs.is_unbound() {
            return true;
        }

        let mut left_iter = self.items();
        let mut right_iter = rhs.items();

        let mut left = left_iter.next();
        let mut right = right_iter.next();

        loop {
            match (left, right) {
                (Some(l), Some(r)) => {
                    if l.start_pos() >= r.start_pos() {
                        if l.start_pos() < r.end_pos() {
                            return true;
                        } else {
                            right = right_iter.next();
                        }
                    } else {
                        if l.end_pos() > r.start_pos() {
                            return true;
                        } else {
                            left = left_iter.next();
                        }
                    }
                }

                _ => break,
            }
        }

        false
    }
}

/// A range between point A and B, `start` and `end` are both std [`Bound`](Bound) objects
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Range<T: Ord> {
    start: Bound<T>,
    end: Bound<T>,
}

impl<T: Ord> From<(T, T)> for Range<T> {
    fn from(value: (T, T)) -> Self {
        Range {
            start: Included(value.0),
            end: Excluded(value.1),
        }
    }
}

impl<T: Ord> Range<T> {
    /// Create a new range from the 2 given bounds
    pub fn new(from: Bound<T>, to: Bound<T>) -> Range<T> {
        Self {
            start: from,
            end: to,
        }
    }

    /// Create a new infinite range
    pub fn unbound() -> Range<T> {
        Self {
            start: Unbounded,
            end: Unbounded,
        }
    }

    /// Return the starting boundary of the range, with the child object as reference
    #[inline]
    pub fn start(&self) -> Bound<&T> {
        self.start.as_ref()
    }

    /// Return the starting boundary or the range but as `PositionalBound`
    #[inline]
    pub fn start_pos(&self) -> PositionalBound<&T> {
        PositionalBound::Start(self.start())
    }

    /// Return the ending boundary of the range, with the child object as reference
    #[inline]
    pub fn end(&self) -> Bound<&T> {
        self.end.as_ref()
    }

    /// Return the ending boundary or the range but as `PositionalBound`
    #[inline]
    pub fn end_pos(&self) -> PositionalBound<&T> {
        PositionalBound::End(self.end())
    }

    /// Returns `true` if this boundary is unbounded, or infinite
    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.start == Unbounded && self.end == Unbounded
    }

    /// Returns `true` if given item falls within this range
    #[inline]
    pub fn contains(&self, item: &T) -> bool {
        (self.start_pos() < item) && (self.end_pos() > item)
    }

    /// Returns the internal `start` and `end` boundaries
    #[inline]
    pub fn into_inner(self) -> (Bound<T>, Bound<T>) {
        (self.start, self.end)
    }
}

impl<T: Ord + Clone> Range<T> {
    /// Create a new `Range` from the
    pub fn from_range<R: RangeBounds<T>>(value: R) -> Self {
        Range {
            start: value.start_bound().cloned(),
            end: value.end_bound().cloned(),
        }
    }
}

///
/// A small wrapper around the std [`Bound`](Bound) enum, that stores if the boundary is the start or end boundary
///
/// This allows us to implement `Ord`, since `Excluded` and `Included` function differentially, in either start or end position
///
#[derive(Debug, Eq, PartialEq)]
pub enum PositionalBound<T> {
    Start(Bound<T>),
    End(Bound<T>),
}

impl<T: Ord> Deref for PositionalBound<T> {
    type Target = Bound<T>;

    fn deref(&self) -> &Self::Target {
        match self {
            PositionalBound::Start(s) => s,
            PositionalBound::End(e) => e,
        }
    }
}

impl<T: Ord + Debug> PartialOrd for PositionalBound<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Debug> Ord for PositionalBound<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PositionalBound::Start(left), PositionalBound::End(right)) => {
                match (left, right) {
                    (Included(left), Included(right)) if left <= right => Ordering::Less,
                    (Included(_), Included(_)) => Ordering::Greater,
                    (Included(left), Excluded(right)) => left.cmp(right),
                    (Excluded(left), Included(right)) if left >= right => Ordering::Greater,
                    (Excluded(left), Excluded(right)) if left >= right => Ordering::Greater,
                    _ => Ordering::Less,
                }
            }

            (PositionalBound::Start(left), PositionalBound::Start(right)) => {
                match (left, right) {
                    (left, right) if left == right => Ordering::Equal,
                    (Unbounded, _) => Ordering::Less,
                    (_, Unbounded) => Ordering::Greater,
                    (Included(left), Included(right)) | (Excluded(left), Excluded(right)) => left.cmp(right),
                    (Included(left), Excluded(right)) if left > right => Ordering::Greater,
                    (Excluded(left), Included(right)) if left >= right => Ordering::Greater,
                    _ => Ordering::Less,
                }
            }

            (PositionalBound::End(left), PositionalBound::Start(right)) => {
                match (left, right) {
                    (Included(left), Included(right)) if left < right => Ordering::Less,
                    (Included(_), Included(_)) => Ordering::Greater,
                    (Included(left), Excluded(right)) if left > right => Ordering::Greater,
                    (Included(_), Excluded(_)) => Ordering::Less,
                    (Excluded(left), Included(right)) => left.cmp(right),
                    (Excluded(left), Excluded(right)) if left <= right => Ordering::Less,
                    _ => Ordering::Greater,
                }
            }

            (PositionalBound::End(left), PositionalBound::End(right)) => {
                match (left, right) {
                    (left, right) if left == right => Ordering::Equal,
                    (Unbounded, _) => Ordering::Greater,
                    (_, Unbounded) => Ordering::Less,
                    (Included(left), Included(right)) | (Excluded(left), Excluded(right)) => left.cmp(right),
                    (Included(left), Excluded(right)) if left >= right => Ordering::Greater,
                    (Excluded(left), Included(right)) if left > right => Ordering::Greater,
                    _ => Ordering::Less,
                }
            }
        }
    }
}


impl<T: Ord> PartialEq<T> for PositionalBound<T> {
    fn eq(&self, _: &T) -> bool {
        // boundaries always fall between atoms
        false
    }
}

impl<T: Ord> PartialOrd<T> for PositionalBound<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        let cmp = match self {
            PositionalBound::Start(Unbounded) => Ordering::Less,
            PositionalBound::End(Unbounded) => Ordering::Greater,
            PositionalBound::Start(Included(left)) | PositionalBound::End(Excluded(left)) if left <= other => Ordering::Less,
            PositionalBound::End(Included(left)) | PositionalBound::Start(Excluded(left)) if left < other => Ordering::Less,
            _ => Ordering::Greater,
        };

        Some(cmp)
    }
}

/// An extension trait for [`Bound`](Bound), implements [`as_ref`](BoundExt::as_ref) and [`invert`](BoundExt::invert)
pub trait BoundExt<T: Ord> {
    /// Invert the position of this boundary
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::ops::Bound::{Excluded, Included, Unbounded};
    /// use eater_rangeset::BoundExt;
    ///
    /// assert_eq!(Excluded(1), Included(1).invert());
    /// assert_eq!(Included(1), Excluded(1).invert());
    /// assert_eq!(Unbounded::<usize>, Unbounded.invert());
    /// ```
    fn invert(self) -> Bound<T>;

    /// Converts `&Bound<T>` into `Bound<&T>`, See also [`Option::as_ref`](Option::as_ref)
    fn as_ref(&self) -> Bound<&T>;
}

impl<T: Ord> BoundExt<T> for Bound<T> {
    #[inline]
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
}

#[cfg(test)]
mod tests {
    use crate::Bound::{Excluded, Included};
    use super::*;

    macro_rules! assert_cmp {
        ($left:expr, $right:expr, $cmp:expr) => {
            actual_assert!($left,$right, $cmp);
            actual_assert!($right, $left, $cmp.reverse());
        };
    }

    macro_rules! actual_assert {
        ($left:expr, $right:expr, $cmp:expr) => {
            let res = $left.cmp(&$right);
            assert_eq!(res, $cmp, "Failed to assert that {:?} <=> {:?} = {:?}", $left, $right, $cmp)
        };
    }

    #[test]
    fn position_bounds() {
        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::End(Included(1)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Included(2)), PositionalBound::End(Included(1)), Ordering::Greater);
        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::End(Excluded(1)), Ordering::Equal);
        assert_cmp!(PositionalBound::Start(Excluded(1)), PositionalBound::End(Included(1)), Ordering::Greater);
        assert_cmp!(PositionalBound::Start(Excluded(0)), PositionalBound::End(Included(1)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Excluded(4)), PositionalBound::End(Excluded(1)), Ordering::Greater);

        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::Start(Included(1)), Ordering::Equal);
        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::Start(Included(2)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::Start(Excluded(1)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Included(1)), PositionalBound::Start(Excluded(2)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Excluded(1)), PositionalBound::Start(Excluded(2)), Ordering::Less);
        assert_cmp!(PositionalBound::Start(Excluded(1)), PositionalBound::Start(Included(2)), Ordering::Less);

        assert_cmp!(PositionalBound::<usize>::Start(Unbounded), PositionalBound::<usize>::Start(Unbounded), Ordering::Equal);

        assert_eq!(PositionalBound::<usize>::Start(Unbounded) < PositionalBound::<usize>::Start(Unbounded), false);

        assert_ne!(PositionalBound::Start(Excluded(1)), 1);

        assert!(PositionalBound::Start(Excluded(4)) < 5);
        assert!(PositionalBound::End(Included(4)) < 5);
    }

    #[test]
    fn contains() {
        let r = range_set!(r!(4..));

        assert!(r.contains(&4));

        let r = range_set!(r!(0..3), r!(4..5));
        assert!(r.contains(&4));
        // Overshoot the first, undershoot the last
        assert!(!r.contains(&3));

        assert!(!r!(0..3).contains(&3));
        assert!(r!(0..3).contains(&0));
    }

    #[test]
    fn add() {
        let mut range = range_set![r!(4..8)];
        range.add(Range::unbound());
        assert_eq!(range, RangeSet::unbound());
        range.add(r!(5..1234));
        assert_eq!(range, RangeSet::unbound());

        let mut range = range_set![];
        range.add(r!(4..10));
        assert_eq!(range_set![r!(4..10)], range);

        let mut range = range_set![r!(20..54)];
        range.add(r!(3..10));

        assert_eq!(range_set![r!(3..10), r!(20..54)], range);

        let mut range = range_set![r!(5..)];
        range.add(r!(1..3));
        assert_eq!(range_set![r!(1..3), r!(5..)], range);

        let mut range = range_set![r!(5..)];
        range.add(r!(1..));
        assert_eq!(range_set![r!(1..)], range);
    }

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
        assert_eq!(test.invert(), expected);
        assert!(test.invert().contains(&4));
    }

    #[test]
    fn intersection() {
        let left: RangeSet<usize> = RangeSet::unbound();
        let right: RangeSet<usize> = RangeSet::from([(Unbounded, Unbounded)]);

        let intersection = left.intersection(&right);
        assert_eq!(left, intersection);

        let left: RangeSet<usize> = range_set!(r!(4..10));
        let right: RangeSet<usize> = range_set!(r!(20..50));
        let intersection = left.intersection(&right);
        assert_eq!(RangeSet::empty(), intersection);
    }

    #[test]
    fn difference() {
        let left: RangeSet<usize> = RangeSet::unbound();
        let right: RangeSet<usize> = RangeSet::empty();

        let diff = left.invert().union(&right).invert();
        assert_eq!(RangeSet::unbound(), diff);

        let left: RangeSet<usize> = RangeSet::unbound();
        let right: RangeSet<usize> = RangeSet::from([1..=4]);

        let expected: RangeSet<usize> = range_set!(r!(..1), r!(4 >..));
        assert_eq!(expected, left.difference(&right));

        let left: RangeSet<usize> = range_set!(r!(..5), r!(8..));
        let right: RangeSet<usize> = range_set!(r!(3..10));

        let expected = range_set!(r!(..3), r!(10..));
        assert_eq!(expected, left.difference(&right));
    }

    #[test]
    fn is_disjoint() {
        let left: RangeSet<usize> = range_set!();
        let right: RangeSet<usize> = range_set!();

        assert!(left.is_disjoint(&right));
        assert!(right.is_disjoint(&left));

        let left: RangeSet<usize> = range_set!(r!(..));

        assert!(left.is_disjoint(&right));
        assert!(right.is_disjoint(&left));

        let left: RangeSet<usize> = range_set!(r!(..));
        let right: RangeSet<usize> = range_set!(r!(..));

        assert!(!left.is_disjoint(&right));
        assert!(!right.is_disjoint(&left));

        let left: RangeSet<usize> = range_set!(r!(1..5), r!(6..20));
        let right: RangeSet<usize> = range_set!(r!(7..10));

        assert!(!left.is_disjoint(&right));
        assert!(!right.is_disjoint(&left));

        let left: RangeSet<usize> = range_set!(r!(1..3), r!(8..20));
        let right: RangeSet<usize> = range_set!(r!(3..6), r!(54..));

        assert!(left.is_disjoint(&right));
        assert!(right.is_disjoint(&left));
    }

    #[test]
    fn is_overlapping() {
        let left: RangeSet<usize> = range_set!();
        let right: RangeSet<usize> = range_set!();

        assert!(!left.is_overlapping(&right));
        assert!(!right.is_overlapping(&left));

        let left: RangeSet<usize> = range_set!(r!(..));

        assert!(!left.is_overlapping(&right));
        assert!(!right.is_overlapping(&left));

        let left: RangeSet<usize> = range_set!(r!(..));
        let right: RangeSet<usize> = range_set!(r!(..));

        assert!(left.is_overlapping(&right));
        assert!(right.is_overlapping(&left));

        let left: RangeSet<usize> = range_set!(r!(1..5), r!(6..20));
        let right: RangeSet<usize> = range_set!(r!(7..10));

        assert!(right.is_overlapping(&left));
        assert!(left.is_overlapping(&right));

        let left: RangeSet<usize> = range_set!(r!(1..3), r!(8..20));
        let right: RangeSet<usize> = range_set!(r!(3..6), r!(54..));

        assert!(!left.is_overlapping(&right));
        assert!(!right.is_overlapping(&left));
    }
}
