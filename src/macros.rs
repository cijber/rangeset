/// Create a new range set based on given ranges
///
/// `<type>:` can be used to set the type when it can't resolved from context
///
/// Examples:
///
/// ```rust
/// use eater_rangeset::{r, range_set};
///
/// let ranges = range_set![r!(..4), r!(10>..)];
/// let ranges = range_set![isize: r!(..)];
/// ```
#[macro_export]
macro_rules! range_set {
    [$ty:ty: $($item:expr),*] => {
        {
            let arr: [$crate::Range<$ty>; 0 $(+ range_set!(([one]) $item))*] = [$($item),*];
            $crate::RangeSet::from(arr)
        }
    };

    [$($item:expr),*] => {
        {
            let arr: [$crate::Range<_>; 0 $(+ range_set!(([one]) $item))*] = [$($item),*];
            $crate::RangeSet::from(arr)
        }
    };

    (([one]) $_item:expr) => {
        1
    }
}


/// Create a new range alike the range operator (`range` is an alias for `r`)
///
/// - `>` can be prefixed to make the start `Exclusive`
/// - `=` can be suffixed to make the end `Inclusive`
///
/// **Note:** an expression in the left position must be wrapped in parenthesis, for rust not to be confused
///
/// # Examples
///
/// ```rust
/// use eater_rangeset::{r, Range, range};
///
/// // Unbound range
/// let a: Range<usize> = r!(..);
///
/// // Exclusive starting range from 4 to infinite
/// let a = r!(4>..);
/// assert_eq!(false, a.contains(&4));
/// let a = r!(..=4);
/// assert_eq!(true, a.contains(&4));
/// let a = range!(..4);
/// assert_eq!(false, a.contains(&4));
/// let a = r!(1>..=4);
/// assert_eq!(true, a.contains(&4));
/// assert_eq!(false, a.contains(&1));
///
/// // Expression start
/// let a = r!((5 + 5) >..);
/// assert_eq!(false, a.contains(&10));
/// assert_eq!(true, a.contains(&11));
/// ```
///
#[macro_export]
macro_rules! r {
    (..) => {
        $crate::Range::new($crate::Bound::Unbounded, $crate::Bound::Unbounded)
    };

    (..$r:expr) => {
        $crate::Range::new($crate::Bound::Unbounded, $crate::Bound::Excluded($r))
    };

    (..= $r:expr) => {
        $crate::Range::new($crate::Bound::Unbounded, $crate::Bound::Included($r))
    };

    ($l:literal >..) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Unbounded)
    };

    ($l:literal >.. $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Excluded($r))
    };

    ($l:literal >..= $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Included($r))
    };

    ($l:literal >.. $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Excluded($r))
    };

    ($l:literal >..= $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Included($r))
    };

    ($l:literal ..) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Unbounded)
    };

    ($l:literal .. $r:expr) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Excluded($r))
    };

    ($l:literal ..= $r:expr) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Included($r))
    };
    
    (($l:expr) >..) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Unbounded)
    };

    (($l:expr) >.. $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Excluded($r))
    };

    (($l:expr) >..= $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Included($r))
    };

    (($l:expr) >.. $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Excluded($r))
    };

    (($l:expr) >..= $r:expr) => {
        $crate::Range::new($crate::Bound::Excluded($l), $crate::Bound::Included($r))
    };

    (($l:expr) ..) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Unbounded)
    };

    (($l:expr) .. $r:expr) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Excluded($r))
    };

    (($l:expr) ..= $r:expr) => {
        $crate::Range::new($crate::Bound::Included($l), $crate::Bound::Included($r))
    };
}