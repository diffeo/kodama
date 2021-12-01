use crate::condensed::CondensedMatrix;
use crate::dendrogram::Dendrogram;
use crate::float::Float;
use crate::method;
use crate::{LinkageState, Method};

/// Perform hierarchical clustering using Müllner's "generic" algorithm.
///
/// In general, one should prefer to use
/// [`linkage`](fn.linkage.html),
/// since it tries to pick the fastest algorithm depending on the method
/// supplied.
pub fn generic<T: Float>(
    condensed_dissimilarity_matrix: &mut [T],
    observations: usize,
    method: Method,
) -> Dendrogram<T> {
    let matrix = condensed_dissimilarity_matrix;
    let mut state = LinkageState::new();
    let mut steps = Dendrogram::new(observations);
    generic_with(&mut state, matrix, observations, method, &mut steps);
    steps
}

/// Like [`generic`](fn.generic.html), but amortizes allocation.
///
/// See [`linkage_with`](fn.linkage_with.html) for details.
#[inline(never)]
pub fn generic_with<T: Float>(
    state: &mut LinkageState<T>,
    condensed_dissimilarity_matrix: &mut [T],
    observations: usize,
    method: Method,
    steps: &mut Dendrogram<T>,
) {
    let matrix = condensed_dissimilarity_matrix;
    method.square(matrix);
    let mut dis = CondensedMatrix::new(matrix, observations);

    steps.reset(dis.observations());
    if dis.observations() == 0 {
        return;
    }
    state.reset(dis.observations());

    {
        // For each observation `row`, find its nearest neighbor and
        // record it in our heap.
        let nearest = &mut state.nearest;
        state.queue.heapify(|dists| {
            for row in 0..dis.observations() - 1 {
                let (mut min, mut min_dist) = (row + 1, dis[[row, row + 1]]);
                for col in (row + 1)..dis.observations() {
                    if dis[[row, col]] < min_dist {
                        min = col;
                        min_dist = dis[[row, col]];
                    }
                }
                dists[row] = min_dist;
                nearest[row] = min;
            }
        });
    }
    for _ in 0..dis.observations() - 1 {
        loop {
            // `a` is our candidate observation to start with. Ideally,
            // state.nearest[a] will tell us which cluster to merge with,
            // but it could be wrong. It is wrong precisely when
            // the minimum distance associated with `a` (its "priority")
            // is less than dis[[a, state.nearest[a]]]. In that case, we need
            // to rescan the other observations and find its actual nearest
            // neighbor.
            let a = state.queue.peek().unwrap();
            if dis[[a, state.nearest[a]]] == *state.queue.priority(a) {
                break;
            }

            let mut min = T::max_value();
            for x in state.active.range(a..).skip(1) {
                if dis[[a, x]] < min {
                    min = dis[[a, x]];
                    state.nearest[a] = x;
                }
            }
            state.queue.set_priority(a, min);
        }

        let a = state.queue.pop().unwrap();
        let b = state.nearest[a];
        let dist = dis[[a, b]];
        match method {
            Method::Single => single(state, &mut dis, a, b),
            Method::Complete => complete(state, &mut dis, a, b),
            Method::Average => average(state, &mut dis, a, b),
            Method::Weighted => weighted(state, &mut dis, a, b),
            Method::Ward => ward(state, &mut dis, a, b),
            Method::Centroid => centroid(state, &mut dis, a, b),
            Method::Median => median(state, &mut dis, a, b),
        }
        state.merge(steps, a, b, dist);
    }
    state.set.relabel(steps, method);
    method.sqrt(steps);
}

#[inline]
fn single<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;

    for x in state.active.range(..a) {
        method::single(dis[[x, a]], &mut dis[[x, b]]);
        if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::single(dis[[a, x]], &mut dis[[x, b]]);
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::single(dis[[a, x]], &mut dis[[b, x]]);
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[inline]
fn complete<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;

    for x in state.active.range(..a) {
        method::complete(dis[[x, a]], &mut dis[[x, b]]);
        if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::complete(dis[[a, x]], &mut dis[[x, b]]);
    }
    for x in state.active.range(b..).skip(1) {
        method::complete(dis[[a, x]], &mut dis[[b, x]]);
    }
}

#[inline]
fn average<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;
    let (size_a, size_b) = (state.sizes[a], state.sizes[b]);

    for x in state.active.range(..a) {
        method::average(dis[[x, a]], &mut dis[[x, b]], size_a, size_b);
        if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::average(dis[[a, x]], &mut dis[[x, b]], size_a, size_b);
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::average(dis[[a, x]], &mut dis[[b, x]], size_a, size_b);
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[inline]
fn weighted<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;

    for x in state.active.range(..a) {
        method::weighted(dis[[x, a]], &mut dis[[x, b]]);
        if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::weighted(dis[[a, x]], &mut dis[[x, b]]);
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::weighted(dis[[a, x]], &mut dis[[b, x]]);
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[inline]
fn ward<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;
    let (size_a, size_b) = (state.sizes[a], state.sizes[b]);
    let dist = dis[[a, b]];

    for x in state.active.range(..a) {
        method::ward(
            dis[[x, a]],
            &mut dis[[x, b]],
            dist,
            size_a,
            size_b,
            state.sizes[x],
        );
        if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
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
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::ward(
            dis[[a, x]],
            &mut dis[[b, x]],
            dist,
            size_a,
            size_b,
            state.sizes[x],
        );
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[inline]
fn centroid<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;
    let (size_a, size_b) = (state.sizes[a], state.sizes[b]);
    let dist = dis[[a, b]];

    for x in state.active.range(..a) {
        method::centroid(dis[[x, a]], &mut dis[[x, b]], dist, size_a, size_b);
        if &dis[[x, b]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, b]]);
            state.nearest[x] = ab;
        } else if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::centroid(dis[[a, x]], &mut dis[[x, b]], dist, size_a, size_b);
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::centroid(dis[[a, x]], &mut dis[[b, x]], dist, size_a, size_b);
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[inline]
fn median<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let ab = b;
    let dist = dis[[a, b]];

    for x in state.active.range(..a) {
        method::median(dis[[x, a]], &mut dis[[x, b]], dist);
        if &dis[[x, b]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, b]]);
            state.nearest[x] = ab;
        } else if state.nearest[x] == a {
            state.nearest[x] = ab;
        }
    }
    for x in state.active.range(a..b).skip(1) {
        method::median(dis[[a, x]], &mut dis[[x, b]], dist);
        if &dis[[x, ab]] < state.queue.priority(x) {
            state.queue.set_priority(x, dis[[x, ab]]);
            state.nearest[x] = ab;
        }
    }
    let mut min = *state.queue.priority(b);
    for x in state.active.range(b..).skip(1) {
        method::median(dis[[a, x]], &mut dis[[b, x]], dist);
        if dis[[ab, x]] < min {
            state.queue.set_priority(b, dis[[ab, x]]);
            state.nearest[b] = x;
            min = dis[[ab, x]];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::generic;
    use crate::test::DistinctMatrix;
    use crate::{nnchain, primitive, Method, MethodChain};

    quickcheck::quickcheck! {
        fn prop_generic_single_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Single);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Single);
            dend_prim == dend_generic
        }

        fn prop_generic_complete_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Complete);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Complete);
            dend_prim == dend_generic
        }

        fn prop_generic_average_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Average);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Average);
            dend_prim == dend_generic
        }

        fn prop_generic_weighted_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Weighted);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Weighted);
            dend_prim == dend_generic
        }

        fn prop_generic_ward_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Ward);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Ward);
            dend_prim == dend_generic
        }

        fn prop_generic_centroid_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Centroid);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Centroid);
            dend_prim.eq_with_epsilon(&dend_generic, 0.0000000001)
        }

        fn prop_generic_median_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Median);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Median);
            dend_prim.eq_with_epsilon(&dend_generic, 0.0000000001)
        }

        fn prop_generic_single_nnchain(mat: DistinctMatrix) -> bool {
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Single);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Single);
            dend_nnchain == dend_generic
        }

        fn prop_generic_complete_nnchain(mat: DistinctMatrix) -> bool {
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Complete);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Complete);
            dend_nnchain == dend_generic
        }

        fn prop_generic_average_nnchain(mat: DistinctMatrix) -> bool {
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Average);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Average);
            dend_nnchain.eq_with_epsilon(&dend_generic, 0.0000000001)
        }

        fn prop_generic_weighted_nnchain(mat: DistinctMatrix) -> bool {
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Weighted);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Weighted);
            dend_nnchain.eq_with_epsilon(&dend_generic, 0.0000000001)
        }

        fn prop_generic_ward_nnchain(mat: DistinctMatrix) -> bool {
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Ward);
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Ward);
            dend_nnchain.eq_with_epsilon(&dend_generic, 0.0000000001)
        }
    }

    /*
    #[test]
    fn scratch() {
        let mat = DistinctMatrix::new(vec![
            -0.3928520988346005, -0.19043623101168627, -0.2285449764242029,
            -0.10865130150304014, -0.15117962251653982, 0.18559742613333996,
            0.4048708395026879, -0.30910143021593295, -0.3052958564360515,
            0.25927061422572684, -0.07587885646654446, 0.22119746284084774,
            0.45489623801622536, -0.27321164703150713, -0.31500339748457495,
        ]);
        let dend_prim = primitive(
            &mut mat.matrix(), mat.len(), Method::Centroid);
        let dend_generic = generic(
            &mut mat.matrix(), mat.len(), Method::Centroid);
        assert_eq!(dend_prim, dend_generic);
    }
    */
}
