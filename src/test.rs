use std::collections::BTreeSet;

use quickcheck::{Arbitrary, Gen};
use rand::Rng;

/// A reflexive pairwise dissimilarity matrix where every dissimilarity is
/// unique.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DistinctMatrix {
    matrix: Vec<f64>,
    len: usize,
}

impl DistinctMatrix {
    /// Create a new matrix from an arbitrary sequence of floating point
    /// numbers. If the given matrix has too many numbers, then it is truncated
    /// to an appropriate length such that it is a reflexive pairwise
    /// dissimilarity matrix.
    ///
    /// Also, any NaN values in the matrix are replaced with `0`.
    pub fn new(mut mat: Vec<f64>) -> DistinctMatrix {
        make_distinct(&mut mat);

        if !mat.is_empty() {
            let mut n = observations(mat.len());
            let mut should = (n * (n - 1)) / 2;
            while should > mat.len() {
                n -= 1;
                should = (n * (n - 1)) / 2;
            }
            mat.truncate(should);

            // Forcefully avoid NaN values. This is consistent with our public
            // API precondition that NaN values aren't permitted.
            for v in &mut mat {
                if v.is_nan() {
                    *v = 0.0;
                }
            }
        }

        let n = observations(mat.len());
        DistinctMatrix { matrix: mat, len: n }
    }

    /// Return a copy of the condensed pairwise dissimilarity matrix.
    pub fn matrix(&self) -> Vec<f64> {
        self.matrix.to_vec()
    }

    /// Return the number of observations in this matrix.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Arbitrary for DistinctMatrix {
    fn arbitrary(_g: &mut Gen) -> DistinctMatrix {
        let mut rng = rand::thread_rng();
        let size = rng.gen_range(0..30);
        let mut dis = vec![];
        for i in 0..size {
            for _ in i + 1..size {
                dis.push(rng.gen_range(-0.5..=0.5));
            }
        }
        DistinctMatrix::new(dis)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = DistinctMatrix>> {
        Box::new(self.matrix.shrink().map(DistinctMatrix::new))
    }
}

/// Mutate `xs` in place such that all of its elements are distinct.
///
/// This will never change the length of `xs` but may change the values of
/// elements in `xs` that are duplicates of other values.
fn make_distinct(xs: &mut Vec<f64>) {
    use std::cmp::Ordering;

    /// NonNanF64 is a wrapper type for floating point types that always
    /// panics during a comparison if the underlying float is a NaN. This
    /// permits us to use floating point numbers as keys in a BTreeSet.
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    struct NonNanF64(f64);

    impl Eq for NonNanF64 {}

    impl Ord for NonNanF64 {
        fn cmp(&self, other: &NonNanF64) -> Ordering {
            self.0.partial_cmp(&other.0).unwrap()
        }
    }

    if xs.is_empty() {
        return;
    }
    // Get the first unique value by adding `1.0` to the max of `xs`.
    let mut next =
        1.0 + xs.iter().fold(xs[0], |a, &b| if a > b { a } else { b });
    let mut seen = BTreeSet::new();
    for i in 0..xs.len() {
        let x = NonNanF64(xs[i]);
        if !seen.contains(&x) {
            seen.insert(x);
            continue;
        }
        xs[i] = next;
        next += 1.0;
    }
}

/// Return an upper bound on the expected number of observations for a given
/// condensed matrix size.
///
/// Note that the size may be invalid. For example, a condensed matrix of
/// size `2` isn't valid.
fn observations(condensed_matrix_size: usize) -> usize {
    ((condensed_matrix_size as f64) * 2.0).sqrt().ceil() as usize
}
