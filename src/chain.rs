use std::mem;

use crate::condensed::CondensedMatrix;
use crate::dendrogram::Dendrogram;
use crate::float::Float;
use crate::method;
use crate::{LinkageState, MethodChain};

/// Perform hierarchical clustering using the "nearest neighbor chain"
/// algorithm as described in Müllner's paper.
///
/// In general, one should prefer to use
/// [`linkage`](fn.linkage.html),
/// since it tries to pick the fastest algorithm depending on the method
/// supplied.
pub fn nnchain<T: Float>(
    dis: &mut [T],
    observations: usize,
    method: MethodChain,
) -> Dendrogram<T> {
    let mut state = LinkageState::new();
    let mut steps = Dendrogram::new(observations);
    nnchain_with(&mut state, dis, observations, method, &mut steps);
    steps
}

/// Like [`nnchain`](fn.nnchain.html), but amortizes allocation.
///
/// See [`linkage_with`](fn.linkage_with.html) for details.
#[inline(never)]
pub fn nnchain_with<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut [T],
    observations: usize,
    method: MethodChain,
    steps: &mut Dendrogram<T>,
) {
    method.square(dis);
    let mut dis = CondensedMatrix::new(dis, observations);

    steps.reset(dis.observations());
    if dis.observations() == 0 {
        return;
    }
    state.reset(dis.observations());
    let (mut a, mut b, mut min);
    state.chain.clear();

    for _ in 0..dis.observations() - 1 {
        if state.chain.len() < 4 {
            a = state
                .active
                .iter()
                .next()
                .expect("at least one active observation");
            state.chain.clear();
            state.chain.push(a);

            b = state.active.iter().nth(1).unwrap();
            min = dis[[a, b]];
            for i in state.active.range(b..).skip(1) {
                if dis[[a, i]] < min {
                    min = dis[[a, i]];
                    b = i;
                }
            }
        } else {
            // All of these unwraps are guaranteed to succeed because
            // state.chain has at least 4 elements.
            state.chain.pop().unwrap();
            state.chain.pop().unwrap();
            b = state.chain.pop().unwrap();
            a = state.chain[state.chain.len() - 1];

            if a < b {
                min = dis[[a, b]];
            } else {
                min = dis[[b, a]];
            }
        }
        loop {
            state.chain.push(b);
            for x in state.active.range(..b) {
                if dis[[x, b]] < min {
                    min = dis[[x, b]];
                    a = x;
                }
            }
            for x in state.active.range(b..).skip(1) {
                if dis[[b, x]] < min {
                    min = dis[[b, x]];
                    a = x;
                }
            }
            b = a;
            a = state.chain[state.chain.len() - 1];
            if b == state.chain[state.chain.len() - 2] {
                break;
            }
        }
        if a > b {
            mem::swap(&mut a, &mut b);
        }
        match method {
            MethodChain::Single => single(state, &mut dis, a, b),
            MethodChain::Complete => complete(state, &mut dis, a, b),
            MethodChain::Average => average(state, &mut dis, a, b),
            MethodChain::Weighted => weighted(state, &mut dis, a, b),
            MethodChain::Ward => ward(state, &mut dis, a, b),
        }
        state.merge(steps, a, b, min);
    }
    state.set.relabel(steps, method.into_method());
    method.sqrt(steps);
}

#[inline]
fn single<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
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

#[inline]
fn complete<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
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

#[inline]
fn average<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let (size_a, size_b) = (state.sizes[a], state.sizes[b]);

    for x in state.active.range(..a) {
        method::average(dis[[x, a]], &mut dis[[x, b]], size_a, size_b);
    }
    for x in state.active.range(a..b).skip(1) {
        method::average(dis[[a, x]], &mut dis[[x, b]], size_a, size_b);
    }
    for x in state.active.range(b..).skip(1) {
        method::average(dis[[a, x]], &mut dis[[b, x]], size_a, size_b);
    }
}

#[inline]
fn weighted<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
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

#[inline]
fn ward<T: Float>(
    state: &mut LinkageState<T>,
    dis: &mut CondensedMatrix<'_, T>,
    a: usize,
    b: usize,
) {
    let dist = dis[[a, b]];
    let (size_a, size_b) = (state.sizes[a], state.sizes[b]);

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

#[cfg(test)]
mod tests {
    use super::nnchain;
    use crate::test::DistinctMatrix;
    use crate::{primitive, Method, MethodChain};

    quickcheck::quickcheck! {
        fn prop_nnchain_single_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Single);
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Single);
            dend_prim == dend_nnchain
        }

        fn prop_nnchain_complete_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Complete);
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Complete);
            dend_prim == dend_nnchain
        }

        fn prop_nnchain_average_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Average);
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Average);
            dend_prim.eq_with_epsilon(&dend_nnchain, 0.0000000001)
        }

        fn prop_nnchain_weighted_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Weighted);
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Weighted);
            dend_prim.eq_with_epsilon(&dend_nnchain, 0.0000000001)
        }

        fn prop_nnchain_ward_primitive(mat: DistinctMatrix) -> bool {
            let dend_prim = primitive(
                &mut mat.matrix(), mat.len(), Method::Ward);
            let dend_nnchain = nnchain(
                &mut mat.matrix(), mat.len(), MethodChain::Ward);
            dend_prim.eq_with_epsilon(&dend_nnchain, 0.0000000001)
        }
    }
}
