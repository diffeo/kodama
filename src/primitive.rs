use crate::active::Active;
use crate::condensed::CondensedMatrix;
use crate::dendrogram::Dendrogram;
use crate::float::Float;
use crate::method;
use crate::{LinkageState, Method};

/// Perform hierarchical clustering using the "primitive" algorithm as
/// described in Müllner's paper.
///
/// Note that this implementation is the "naive" implementation of
/// hierarchical clustering, and is therefore terribly slow. Use
/// [`linkage`](fn.linkage.html)
/// instead to have the appropriate algorithm chosen for you.
pub fn primitive<T: Float>(
    dis: &mut [T],
    observations: usize,
    method: Method,
) -> Dendrogram<T> {
    let mut state = LinkageState::new();
    let mut steps = Dendrogram::new(observations);
    primitive_with(&mut state, dis, observations, method, &mut steps);
    steps
}

/// Like [`primitive`](fn.primitive.html), but amortizes allocation.
///
/// See [`linkage_with`](fn.linkage_with.html) for details.
///
/// Note that this implementation is the "naive" implementation of
/// hierarchical clustering, and is therefore terribly slow.
#[inline(never)]
pub fn primitive_with<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut [T],
    observations: usize,
    method: Method,
    steps: &mut Dendrogram<T>,
) {
    method.square(dis);
    let mut dis = CondensedMatrix::new(dis, observations);

    steps.reset(dis.observations());
    if dis.observations() == 0 {
        return;
    }
    state.reset(dis.observations());

    for _ in 0..dis.observations() - 1 {
        let (a, b, dist) = argmin(&dis, &state.active).unwrap();
        let (size_a, size_b) = (state.sizes[a], state.sizes[b]);

        match method {
            Method::Single => {
                for x in state.active.range(..a) {
                    method::single(dis[[x, a]], &mut dis[[x, b]]);
                }
                for x in state.active.range(a..b).skip(1) {
                    method::single(dis[[a, x]], &mut dis[[x, b]]);
                }
                for x in state.active.range(b..).skip(1) {
                    method::single(dis[[a, x]], &mut dis[[b, x]]);
                }
            }
            Method::Complete => {
                for x in state.active.range(..a) {
                    method::complete(dis[[x, a]], &mut dis[[x, b]]);
                }
                for x in state.active.range(a..b).skip(1) {
                    method::complete(dis[[a, x]], &mut dis[[x, b]]);
                }
                for x in state.active.range(b..).skip(1) {
                    method::complete(dis[[a, x]], &mut dis[[b, x]]);
                }
            }
            Method::Average => {
                for x in state.active.range(..a) {
                    method::average(
                        dis[[x, a]],
                        &mut dis[[x, b]],
                        size_a,
                        size_b,
                    );
                }
                for x in state.active.range(a..b).skip(1) {
                    method::average(
                        dis[[a, x]],
                        &mut dis[[x, b]],
                        size_a,
                        size_b,
                    );
                }
                for x in state.active.range(b..).skip(1) {
                    method::average(
                        dis[[a, x]],
                        &mut dis[[b, x]],
                        size_a,
                        size_b,
                    );
                }
            }
            Method::Weighted => {
                for x in state.active.range(..a) {
                    method::weighted(dis[[x, a]], &mut dis[[x, b]]);
                }
                for x in state.active.range(a..b).skip(1) {
                    method::weighted(dis[[a, x]], &mut dis[[x, b]]);
                }
                for x in state.active.range(b..).skip(1) {
                    method::weighted(dis[[a, x]], &mut dis[[b, x]]);
                }
            }
            Method::Ward => {
                for x in state.active.range(..a) {
                    method::ward(
                        dis[[x, a]],
                        &mut dis[[x, b]],
                        dist,
                        size_a,
                        size_b,
                        state.sizes[x],
                    );
                }
                for x in state.active.range(a..b).skip(1) {
                    method::ward(
                        dis[[a, x]],
                        &mut dis[[x, b]],
                        dist,
                        size_a,
                        size_b,
                        state.sizes[x],
                    );
                }
                for x in state.active.range(b..).skip(1) {
                    method::ward(
                        dis[[a, x]],
                        &mut dis[[b, x]],
                        dist,
                        size_a,
                        size_b,
                        state.sizes[x],
                    );
                }
            }
            Method::Centroid => {
                for x in state.active.range(..a) {
                    method::centroid(
                        dis[[x, a]],
                        &mut dis[[x, b]],
                        dist,
                        size_a,
                        size_b,
                    );
                }
                for x in state.active.range(a..b).skip(1) {
                    method::centroid(
                        dis[[a, x]],
                        &mut dis[[x, b]],
                        dist,
                        size_a,
                        size_b,
                    );
                }
                for x in state.active.range(b..).skip(1) {
                    method::centroid(
                        dis[[a, x]],
                        &mut dis[[b, x]],
                        dist,
                        size_a,
                        size_b,
                    );
                }
            }
            Method::Median => {
                for x in state.active.range(..a) {
                    method::median(dis[[x, a]], &mut dis[[x, b]], dist);
                }
                for x in state.active.range(a..b).skip(1) {
                    method::median(dis[[a, x]], &mut dis[[x, b]], dist);
                }
                for x in state.active.range(b..).skip(1) {
                    method::median(dis[[a, x]], &mut dis[[b, x]], dist);
                }
            }
        }
        state.merge(steps, a, b, dist);
    }
    state.set.relabel(steps, method);
    method.sqrt(steps);
}

#[inline(never)]
fn argmin<T: Float>(
    matrix: &CondensedMatrix<'_, T>,
    active: &Active,
) -> Option<(usize, usize, T)> {
    // A natural representation for min is Option<_>, but this requires
    // an additional comparison in the inner loop to check for None.
    // Instead, we use the first active cell in the matrix as our initial
    // minimum.
    let mut min = match active.iter().next() {
        None => return None,
        Some(row) => match active.range(row..).skip(1).next() {
            None => return None,
            Some(col) => (row, col, matrix[[row, col]]),
        },
    };
    for row in active.iter() {
        for col in active.range(row..).skip(1) {
            let value = matrix[[row, col]];
            if value < min.2 {
                min = (row, col, value);
            }
        }
    }
    Some(min)
}

#[cfg(test)]
mod tests {
    use super::argmin;
    use crate::active::Active;
    use crate::condensed::CondensedMatrix;

    #[test]
    fn argmin_zero() {
        let mut data: Vec<f64> = vec![];
        let mat = CondensedMatrix::new(&mut data, 0);
        assert!(argmin(&mat, &Active::with_len(0)).is_none());
    }

    #[test]
    fn argmin_smallest() {
        let mut data = vec![1.0];
        let mat = CondensedMatrix::new(&mut data, 2);
        assert_eq!(argmin(&mat, &Active::with_len(2)).unwrap(), (0, 1, 1.0));
    }

    #[test]
    fn argmin_simple() {
        let mut data = vec![0.1, 0.2, 0.3, 0.4, 1.2, 0.01, 1.4, 2.3, 2.4, 3.4];
        let mat = CondensedMatrix::new(&mut data, 5);
        assert_eq!(argmin(&mat, &Active::with_len(5)).unwrap(), (1, 3, 0.01));
    }
}
