use std::ops::{Index, IndexMut};

/// A simple wrapper for convenient 2-dimensional indexing into a condensed
/// pairwise reflexive dissimilarity matrix.
///
/// This condensed matrix can be conveniently indexed using a 2-element array.
/// For example, if `mat` is a `CondensedMatrix`, then `mat[[2, 5]]` returns
/// the dissimilarity between observations `2` and `5` (where `2` is the row
/// index and `5` is the column index).
///
/// The lifetime `'a` refers to the lifetime of the borrowed matrix.
///
/// # Panics
///
/// Note that this representation uses the upper right triangle of a pairwise
/// reflexive dissimilarity matrix. Therefore, the row index must always be
/// less than the column index. Violating this results in a panic.
#[derive(Debug)]
pub struct CondensedMatrix<'a, T: 'a> {
    data: &'a mut [T],
    observations: usize,
}

impl<'a, T> CondensedMatrix<'a, T> {
    /// Create a new indexable condensed pairwise matrix.
    ///
    /// `data` should be the contiguous condensed pairwise matrix, where each
    /// row of the upper triangle in the matrix are laid out contiguously.
    ///
    /// `observations` should be the number of observations that make up the
    /// matrix.
    ///
    /// # Panics
    ///
    /// This method panics when the length of the matrix given is inconsistent
    /// with the number of observations given. In particular, the following
    /// identity is enforced:
    ///
    /// ```text
    /// data.len() == (observations * (observations - 1)) / 2
    /// ```
    ///
    /// As a special case, if `observations` is `<= 1`, then it is treated as
    /// if it is equivalent to `0`. In this case, the matrix provided must be
    /// empty.
    pub fn new(
        data: &'a mut [T],
        observations: usize,
    ) -> CondensedMatrix<'a, T> {
        if data.is_empty() {
            assert!(observations <= 1);
            CondensedMatrix { data: data, observations: 0 }
        } else {
            assert!(observations >= 2);
            assert_eq!((observations * (observations - 1)) / 2, data.len());
            CondensedMatrix { data: data, observations: observations }
        }
    }

    /// Return the number of observations that make up this matrix.
    pub fn observations(&self) -> usize {
        self.observations
    }

    /// Convert the given row and column 2-dimensional index into an index
    /// into the condensed matrix.
    fn matrix_to_condensed_idx(&self, row: usize, column: usize) -> usize {
        debug_assert!(row < column);
        debug_assert!(column < self.observations());
        // The more natural indexing scheme is probably this:
        //
        //     ((self.observations * row) + column)
        //     - ((row * (row + 1)) / 2)
        //     - 1 - row
        //
        // However, the formulation below seems to do a bit better in ad hoc
        // benchmarks. If you write down the above formula, then it is easy
        // to arrive to the formula below through simple algebraic
        // transformations.
        ((2 * self.observations() - row - 3) * row / 2) + column - 1
    }
}

impl<'a, T> Index<[usize; 2]> for CondensedMatrix<'a, T> {
    type Output = T;

    fn index(&self, idx: [usize; 2]) -> &T {
        &self.data[self.matrix_to_condensed_idx(idx[0], idx[1])]
    }
}

impl<'a, T> IndexMut<[usize; 2]> for CondensedMatrix<'a, T> {
    fn index_mut(&mut self, idx: [usize; 2]) -> &mut T {
        let i = self.matrix_to_condensed_idx(idx[0], idx[1]);
        &mut self.data[i]
    }
}
