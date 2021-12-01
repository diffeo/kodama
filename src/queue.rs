use crate::float::Float;

// BREADCRUMBS: Look into moving `nearest` into this heap structure. It seems
// like it might be doing too much at once, but things might actually become
// clearer by coupling them together since they do actually seem coupled in
// the generic algorithm.

/// A priority queue of clusters implement by a min-binary-heap.
///
/// Elements in the queue are cluster labels, and are weighted by a candidate
/// minimum dissimilarity between it and its nearest cluster.
#[derive(Debug, Default)]
pub struct LinkageHeap<T> {
    /// A heap of observations. A node N has children at 2N+1 and 2N+2.
    heap: Vec<usize>,
    /// A map from observation to its position in `heap`.
    observations: Vec<usize>,
    /// The priority associated with each observation.
    priorities: Vec<T>,
    /// Observations that have been removed.
    removed: Vec<bool>,
}

impl<T: Float> LinkageHeap<T> {
    pub fn new() -> LinkageHeap<T> {
        LinkageHeap::with_len(0)
    }

    pub fn with_len(len: usize) -> LinkageHeap<T> {
        LinkageHeap {
            heap: (0..len).collect(),
            observations: (0..len).collect(),
            priorities: vec![T::max_value(); len],
            removed: vec![false; len],
        }
    }

    pub fn reset(&mut self, len: usize) {
        self.heap.resize(len, 0);
        self.observations.resize(len, 0);
        self.priorities.resize(len, T::max_value());
        self.removed.resize(len, false);

        for i in 0..len {
            self.heap[i] = i;
            self.observations[i] = i;
            self.priorities[i] = T::max_value();
            self.removed[i] = false;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        if self.heap.len() >= 2 {
            let (first, last) = (self.heap[0], *self.heap.last().unwrap());
            self.swap(first, last);
        }
        let last = self.heap.pop().unwrap();
        self.removed[last] = true;
        if self.heap.len() >= 2 {
            let first = self.heap[0];
            self.sift_down(first);
        }
        Some(last)
    }

    pub fn peek(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.heap[0])
        }
    }

    pub fn heapify<F: FnMut(&mut [T])>(&mut self, mut f: F) {
        let len = self.priorities.len();
        self.reset(len);
        f(&mut self.priorities);

        for i in (0..len / 2).rev() {
            let o = self.heap[i];
            self.sift_down(o);
        }
    }

    pub fn priority(&self, observation: usize) -> &T {
        assert!(!self.removed[observation]);
        &self.priorities[observation]
    }

    pub fn set_priority(&mut self, observation: usize, priority: T) {
        assert!(!self.removed[observation]);

        let old = self.priorities[observation];
        self.priorities[observation] = priority;
        if priority < old {
            self.sift_up(observation);
        } else if priority > old {
            self.sift_down(observation);
        }
    }

    fn sift_up(&mut self, o: usize) {
        loop {
            let po = match self.parent(o) {
                None => break,
                Some(po) => po,
            };
            if self.priorities[po] < self.priorities[o] {
                break;
            }
            self.swap(o, po);
        }
    }

    fn sift_down(&mut self, o: usize) {
        loop {
            let mut child = o;
            let (left, right) = self.children(o);
            if let Some(left) = left {
                if self.priorities[left] < self.priorities[child] {
                    child = left;
                }
            }
            if let Some(right) = right {
                if self.priorities[right] < self.priorities[child] {
                    child = right;
                }
            }
            if o == child {
                break;
            }
            self.swap(o, child);
        }
    }

    fn parent(&self, o: usize) -> Option<usize> {
        if self.observations[o] == 0 {
            None
        } else {
            Some(self.heap[(self.observations[o] - 1) / 2])
        }
    }

    fn children(&self, o: usize) -> (Option<usize>, Option<usize>) {
        let i = self.observations[o];
        let (left, right) = (2 * i + 1, 2 * i + 2);
        (self.heap.get(left).cloned(), self.heap.get(right).cloned())
    }

    fn swap(&mut self, o1: usize, o2: usize) {
        self.heap.swap(self.observations[o1], self.observations[o2]);
        self.observations.swap(o1, o2);
    }
}

#[cfg(test)]
mod tests {
    use crate::float::Float;

    use super::LinkageHeap;

    fn is_sorted_asc<T: Float>(xs: &[T]) -> bool {
        for win in xs.windows(2) {
            if win[0] > win[1] {
                return false;
            }
        }
        true
    }

    fn pop_all<T: Float>(heap: &mut LinkageHeap<T>) -> Vec<T> {
        let mut xs = vec![];
        while let Some(o) = heap.peek() {
            xs.push(*heap.priority(o));
            heap.pop().unwrap();
        }
        xs
    }

    fn new_heap<T: Float>(priorities: &[T]) -> LinkageHeap<T> {
        let mut heap = LinkageHeap::with_len(priorities.len());
        for (i, p) in priorities.iter().enumerate() {
            heap.set_priority(i, *p);
        }
        heap
    }

    fn heapify<T: Float>(priorities: &[T]) -> LinkageHeap<T> {
        let mut heap = LinkageHeap::with_len(priorities.len());
        heap.heapify(|ps| ps.copy_from_slice(priorities));
        heap
    }

    #[test]
    fn simple() {
        let mut heap = new_heap(&[2.0, 1.0, 10.0, 5.0, 4.0, 4.5]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0, 2.0, 4.0, 4.5, 5.0, 10.0]);

        let mut heap = heapify(&[2.0, 1.0, 10.0, 5.0, 4.0, 4.5]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0, 2.0, 4.0, 4.5, 5.0, 10.0]);
    }

    #[test]
    fn empty() {
        let mut heap = new_heap::<f64>(&[]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[]);

        let mut heap = heapify::<f64>(&[]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[]);
    }

    #[test]
    fn one() {
        let mut heap = new_heap(&[1.0]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0]);

        let mut heap = heapify(&[1.0]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0]);
    }

    #[test]
    fn two() {
        let mut heap = new_heap(&[2.0, 1.0]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0, 2.0]);

        let mut heap = heapify(&[2.0, 1.0]);
        let ps = pop_all(&mut heap);
        assert_eq!(ps, &[1.0, 2.0]);
    }

    quickcheck::quickcheck! {
        fn prop_heap_invariant(xs: Vec<f64>) -> bool {
            let mut xs = xs;
            for x in &mut xs {
                if x.is_nan() {
                    *x = 0.0;
                }
            }
            let mut heap = new_heap(&xs);
            is_sorted_asc(&pop_all(&mut heap))
        }
    }

    quickcheck::quickcheck! {
        fn prop_heapify_heap_invariant(xs: Vec<f64>) -> bool {
            let mut xs = xs;
            for x in &mut xs {
                if x.is_nan() {
                    *x = 0.0;
                }
            }
            let mut heap = heapify(&xs);
            is_sorted_asc(&pop_all(&mut heap))
        }
    }
}
