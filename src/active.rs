use std::collections::Bound;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

/// Active is a list of contiguous unsigned integers that supports efficient
/// removal and iteration that is proportional to the number of elements in the
/// list.
#[derive(Clone, Debug, Default)]
pub struct Active {
    /// The first active integer.
    start: usize,
    /// prev[i] is the active integer that precedes `i`, where `i` is also
    /// active. prev[i] is unspecified for all inactive `i`.
    prev: Vec<usize>,
    /// next[i] is the active integer that follows `i`, where `i` is also
    /// active. next[i] == 0 for all inactive `i`.
    next: Vec<usize>,
}

impl Active {
    /// Create a new empty active list.
    pub fn new() -> Active {
        Active { start: 0, prev: vec![], next: vec![] }
    }

    /// Create a new active list with elements `0` through `len-1`, inclusive.
    pub fn with_len(len: usize) -> Active {
        let mut a = Active::new();
        a.reset(len);
        a
    }

    /// Reset this list to the given length as if a new list were created.
    ///
    /// This permits reusing this list's allocation.
    pub fn reset(&mut self, len: usize) {
        self.start = 0;
        self.prev.resize(len, 0);
        self.next.resize(len, 0);
        for i in 0..len {
            self.prev[i] = i;
            self.next[i] = i + 1;
        }
        self.start = 0;
    }

    /// Return true if the given element is still in the list.
    ///
    /// This runs in constant time.
    pub fn contains(&self, i: usize) -> bool {
        self.next[i] > 0
    }

    /// Remove the given element from this list.
    ///
    /// If the given element has already been removed, then this is a no-op.
    ///
    /// This runs in constant time.
    pub fn remove(&mut self, i: usize) {
        if !self.contains(i) {
            return;
        }
        if i == self.start {
            self.start = self.next[i];
        } else {
            assert!(i > self.start);
            self.prev[self.next[i] - 1] = self.prev[i - 1];
            self.next[self.prev[i - 1]] = self.next[i];
        }
        // The first item can never be the next item, so we
        // reuse it as a sentinel.
        self.next[i] = 0;
    }

    /// Return an iterator over every element in the list.
    ///
    /// The iterator runs in time proportional to the number of elements in
    /// the list.
    pub fn iter(&self) -> ActiveIter<'_> {
        ActiveIter(ActiveRange {
            active: self,
            cur: self.start,
            end: self.next.len(),
        })
    }

    /// Return an iterator over every element in the list in the range given.
    ///
    /// If the boundaries of the given range correspond to elements in this
    /// list, then the iterator runs in time proportional to the number of
    /// elements in the list in the given range. Otherwise, no such guarantee
    /// is provided, but has an upper bound on the total number of elements
    /// that have ever been in the list.
    pub fn range<R: RangeBound<usize>>(&self, range: R) -> ActiveRange<'_> {
        let mut start = match range.start() {
            Bound::Unbounded => self.start,
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
        };
        let end = match range.end() {
            Bound::Unbounded => self.next.len(),
            Bound::Included(&i) => i + 1,
            Bound::Excluded(&i) => i,
        };
        assert!(start <= self.next.len());
        assert!(end <= self.next.len());

        // The start of our range may not correspond to
        // an active observation, so find the first active
        // observation.
        if start < self.start {
            start = self.start;
        }
        while start < self.next.len() && !self.contains(start) {
            start += 1;
        }
        ActiveRange { active: self, cur: start, end: end }
    }
}

impl<'a> IntoIterator for &'a Active {
    type IntoIter = ActiveIter<'a>;
    type Item = usize;
    fn into_iter(self) -> ActiveIter<'a> {
        self.iter()
    }
}

/// An iterator over all elements in an active list.
///
/// The lifetime `'a` refers to the underlying active list.
#[derive(Clone, Debug)]
pub struct ActiveIter<'a>(ActiveRange<'a>);

impl<'a> Iterator for ActiveIter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        self.0.next()
    }
}

/// An iterator over all elements in an active list in a particular range.
///
/// The lifetime `'a` refers to the underlying active list.
#[derive(Clone, Debug)]
pub struct ActiveRange<'a> {
    active: &'a Active,
    cur: usize,
    end: usize,
}

impl<'a> Iterator for ActiveRange<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        let observation = self.cur;
        if observation >= self.end || observation >= self.active.next.len() {
            return None;
        }
        self.cur = self.active.next[observation];
        Some(observation)
    }
}

/// A trait that abstracts over the different types of ranges.
///
/// We define this ourselves until std::collections::range stabilizes.
pub trait RangeBound<T> {
    /// Return the start bound.
    fn start(&self) -> Bound<&T>;
    /// Return the end bound.
    fn end(&self) -> Bound<&T>;
}

impl<T> RangeBound<T> for RangeFull {
    fn start(&self) -> Bound<&T> {
        Bound::Unbounded
    }
    fn end(&self) -> Bound<&T> {
        Bound::Unbounded
    }
}

impl<T> RangeBound<T> for RangeFrom<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end(&self) -> Bound<&T> {
        Bound::Unbounded
    }
}

impl<T> RangeBound<T> for RangeTo<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Unbounded
    }
    fn end(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

impl<T> RangeBound<T> for Range<T> {
    fn start(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::Active;
    use std::ops::Range;

    fn items(active: &Active) -> Vec<usize> {
        active.iter().collect()
    }

    fn items_range(active: &Active, range: Range<usize>) -> Vec<usize> {
        active.range(range).collect()
    }

    #[test]
    fn contains() {
        let mut a = Active::with_len(10);
        for i in 0..10 {
            assert!(a.contains(i));
        }
        a.remove(0);
        assert!(!a.contains(0));
        a.remove(5);
        assert!(!a.contains(5));
    }

    #[test]
    fn iter() {
        let mut a = Active::with_len(5);
        assert_eq!(items(&a), vec![0, 1, 2, 3, 4]);

        a.remove(2);
        assert_eq!(items(&a), vec![0, 1, 3, 4]);

        a.remove(4);
        assert_eq!(items(&a), vec![0, 1, 3]);

        a.remove(0);
        assert_eq!(items(&a), vec![1, 3]);

        a.remove(3);
        assert_eq!(items(&a), vec![1]);

        a.remove(1);
        assert_eq!(items(&a), vec![]);
    }

    #[test]
    fn iter_range() {
        let mut a = Active::with_len(5);
        assert_eq!(items_range(&a, 0..5), vec![0, 1, 2, 3, 4]);
        assert_eq!(items_range(&a, 0..1), vec![0]);
        assert_eq!(items_range(&a, 1..3), vec![1, 2]);
        assert_eq!(items_range(&a, 2..5), vec![2, 3, 4]);
        assert_eq!(items_range(&a, 3..5), vec![3, 4]);
        assert_eq!(items_range(&a, 4..5), vec![4]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);

        a.remove(2);
        assert_eq!(items_range(&a, 0..5), vec![0, 1, 3, 4]);
        assert_eq!(items_range(&a, 0..1), vec![0]);
        assert_eq!(items_range(&a, 1..3), vec![1]);
        assert_eq!(items_range(&a, 2..5), vec![3, 4]);
        assert_eq!(items_range(&a, 3..5), vec![3, 4]);
        assert_eq!(items_range(&a, 4..5), vec![4]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);

        a.remove(4);
        assert_eq!(items_range(&a, 0..5), vec![0, 1, 3]);
        assert_eq!(items_range(&a, 0..1), vec![0]);
        assert_eq!(items_range(&a, 1..3), vec![1]);
        assert_eq!(items_range(&a, 2..5), vec![3]);
        assert_eq!(items_range(&a, 3..5), vec![3]);
        assert_eq!(items_range(&a, 4..5), vec![]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);

        a.remove(0);
        assert_eq!(items_range(&a, 0..5), vec![1, 3]);
        assert_eq!(items_range(&a, 0..1), vec![]);
        assert_eq!(items_range(&a, 1..3), vec![1]);
        assert_eq!(items_range(&a, 2..5), vec![3]);
        assert_eq!(items_range(&a, 3..5), vec![3]);
        assert_eq!(items_range(&a, 4..5), vec![]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);

        a.remove(3);
        assert_eq!(items_range(&a, 0..5), vec![1]);
        assert_eq!(items_range(&a, 0..1), vec![]);
        assert_eq!(items_range(&a, 1..3), vec![1]);
        assert_eq!(items_range(&a, 2..5), vec![]);
        assert_eq!(items_range(&a, 3..5), vec![]);
        assert_eq!(items_range(&a, 4..5), vec![]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);

        a.remove(1);
        assert_eq!(items_range(&a, 0..5), vec![]);
        assert_eq!(items_range(&a, 0..1), vec![]);
        assert_eq!(items_range(&a, 1..3), vec![]);
        assert_eq!(items_range(&a, 2..5), vec![]);
        assert_eq!(items_range(&a, 3..5), vec![]);
        assert_eq!(items_range(&a, 4..5), vec![]);
        assert_eq!(items_range(&a, 0..0), vec![]);
        assert_eq!(items_range(&a, 1..1), vec![]);
        assert_eq!(items_range(&a, 5..5), vec![]);
    }
}
