use crate::condensed::CondensedMatrix;
use crate::dendrogram::Dendrogram;
use crate::float::Float;
use crate::method;
use crate::{LinkageState, Method};

/// Perform hierarchical clustering using the minimum spanning tree
/// algorithm as described in Müllner's paper.
///
/// In general, one should prefer to use
/// [`linkage`](fn.linkage.html),
/// since it tries to pick the fastest algorithm depending on the method
/// supplied.
pub fn mst<T: Float>(dis: &mut [T], observations: usize) -> Dendrogram<T> {
    let mut state = LinkageState::new();
    let mut steps = Dendrogram::new(observations);
    mst_with(&mut state, dis, observations, &mut steps);
    steps
}

/// Like [`mst`](fn.mst.html), but amortizes allocation.
///
/// See [`linkage_with`](fn.linkage_with.html) for details.
#[inline(never)]
pub fn mst_with<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut [T],
    observations: usize,
    steps: &mut Dendrogram<T>,
) {
    let dis = CondensedMatrix::new(dis, observations);

    steps.reset(dis.observations());
    if dis.observations() == 0 {
        return;
    }
    state.reset(dis.observations());

    let mut cluster = 0;
    state.active.remove(cluster);

    for _ in 0..dis.observations() - 1 {
        let mut min_obs = state
            .active
            .iter()
            .next()
            .expect("at least one active observation");
        let mut min_dist = state.min_dists[min_obs];

        for x in state.active.range(..cluster) {
            let slot = &mut state.min_dists[x];
            method::single(dis[[x, cluster]], slot);
            if *slot < min_dist {
                min_obs = x;
                min_dist = *slot;
            }
        }
        for x in state.active.range(cluster..) {
            let slot = &mut state.min_dists[x];
            method::single(dis[[cluster, x]], slot);
            if *slot < min_dist {
                min_obs = x;
                min_dist = *slot;
            }
        }
        state.merge(steps, min_obs, cluster, min_dist);
        cluster = min_obs;
    }
    state.set.relabel(steps, Method::Single);
}

#[cfg(test)]
mod tests {
    use super::mst;
    use crate::test::DistinctMatrix;
    use crate::{generic, primitive, Method};

    quickcheck::quickcheck! {
        fn prop_mst_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Single);
            let dend_mst = mst(
                &mut mat.matrix(), mat.len());
            dend_prim == dend_mst
        }

        fn prop_mst_generic(mat: DistinctMatrix) -> bool {
            let dend_generic = generic(
                &mut mat.matrix(), mat.len(), Method::Single);
            let dend_mst = mst(
                &mut mat.matrix(), mat.len());
            dend_generic == dend_mst
        }
    }
}
