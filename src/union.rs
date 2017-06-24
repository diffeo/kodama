use std::usize;

use dendrogram::Dendrogram;
use Method;

/// A specialized implementation of union-find for linkage.
///
/// This union-find implementation represents a set of cluster labels. It
/// supports fast lookups and fast unions.
///
/// This specific data structure is design to support cluster labels for a
/// fixed set of observations. Namely, if there are `N` observations, then
/// there are `N + N - 1` possible cluster labels.
#[derive(Clone, Debug)]
pub struct LinkageUnionFind {
    /// A map from cluster label to its cluster's parent.
    ///
    /// When a cluster label is mapped to itself, then it is considered a
    /// root.
    parents: Vec<usize>,
    /// The next cluster label to assign on the next union.
    next_parent: usize,
}

impl Default for LinkageUnionFind {
    fn default() -> LinkageUnionFind { LinkageUnionFind::new() }
}

impl LinkageUnionFind {
    /// Create a new empty set.
    pub fn new() -> LinkageUnionFind {
        LinkageUnionFind::with_len(0)
    }

    /// Create a new set that can merge clusters for exactly `len`
    /// observations.
    pub fn with_len(len: usize) -> LinkageUnionFind {
        let size = if len == 0 { 0 } else { 2 * len - 1 };
        LinkageUnionFind {
            parents: (0..size).collect(),
            next_parent: len,
        }
    }

    /// Clear this allocation and resize it as appropriate to support `len`
    /// observations.
    pub fn reset(&mut self, len: usize) {
        let size = if len == 0 { 0 } else { 2 * len - 1 };
        self.next_parent = len;
        self.parents.resize(size, 0);
        for (i, parent) in self.parents.iter_mut().enumerate() {
            *parent = i;
        }
    }

    /// Union the two clusters represented by the given labels.
    ///
    /// If the two clusters have already been merged, then this is a no-op.
    pub fn union(&mut self, cluster1: usize, cluster2: usize) {
        // If the clusters are already in the same set, then
        // this is a no-op.
        if self.find(cluster1) == self.find(cluster2) {
            return;
        }

        assert!(self.next_parent < self.parents.len());
        self.parents[cluster1] = self.next_parent;
        self.parents[cluster2] = self.next_parent;
        self.next_parent = self.next_parent + 1;
    }

    /// Return the root cluster label containing the cluster given.
    pub fn find(&mut self, mut cluster: usize) -> usize {
        // Find the parent of this cluster. The parent
        // is the "label" of the cluster and is a root
        // element.
        let mut parent = cluster;
        while let Some(p) = self.parent(parent) {
            parent = p;
        }
        // To speed up subsequent calls to `find`, we
        // set the parent of this cluster and all of its
        // ancestors up to `parent`.
        while let Some(p) = self.parent(cluster) {
            self.parents[cluster] = parent;
            cluster = p;
        }
        parent
    }

    /// Return the parent of the given cluster, if one exists. If the given
    /// cluster is a root, then `None` is returned.
    fn parent(&self, cluster: usize) -> Option<usize> {
        let p = self.parents[cluster];
        if p == cluster {
            None
        } else {
            Some(p)
        }
    }

    /// Relabel the cluster labels in each step of a complete dendrogram.
    ///
    /// If the given method requires the dendrogram to be sorted, then the
    /// steps of the dendrogram are sorted by their dissimilarities.
    pub fn relabel<T: PartialOrd>(
        &mut self,
        dendrogram: &mut Dendrogram<T>,
        method: Method,
    ) {
        self.reset(dendrogram.observations());
        if method.requires_sorting() {
            dendrogram.steps_mut().sort_by(|step1, step2| {
                // Floats have a partial ordering because of NaN. There's
                // basically two reasonable things we could do here:
                //
                //   1. Panic if we find a NaN. A NaN dissimilarity between two
                //      clusters probably indicates a bug somewhere, and it's
                //      not clear when this could happen.
                //   2. If we have a NaN. then cast to bit representation and
                //      derive an ordering from that (or follow whatever IEEE
                //      says).
                //
                // We choose door #1 because it likely indicates a bug, and
                // we'd rather the bug fail loudly until we understand it
                // enough that we can fix it some other way.
                step1.dissimilarity.partial_cmp(&step2.dissimilarity)
                    .expect("NaNs not allowed in dendrogram")
            });
        }
        for i in 0..dendrogram.len() {
            let new_cluster1 = self.find(dendrogram[i].cluster1);
            let new_cluster2 = self.find(dendrogram[i].cluster2);
            self.union(new_cluster1, new_cluster2);

            let size1 = dendrogram.cluster_size(new_cluster1);
            let size2 = dendrogram.cluster_size(new_cluster2);
            dendrogram[i].set_clusters(new_cluster1, new_cluster2);
            dendrogram[i].size = size1 + size2;
        }
    }
}

#[cfg(test)]
mod tests {
    use dendrogram::{Dendrogram, Step};
    use super::LinkageUnionFind;
    use Method;

    #[test]
    fn trivial_find() {
        let mut set = LinkageUnionFind::with_len(5);
        // In the trivial set, each member is its own cluster.
        for i in 0..5 {
            assert_eq!(i, set.find(i));
        }
    }

    #[test]
    fn find_with_unions() {
        let mut set = LinkageUnionFind::with_len(5);

        set.union(1, 3);
        assert_eq!(0, set.find(0));
        assert_eq!(5, set.find(1));
        assert_eq!(2, set.find(2));
        assert_eq!(5, set.find(3));
        assert_eq!(4, set.find(4));
        assert_eq!(5, set.find(5));

        set.union(5, 2);
        assert_eq!(0, set.find(0));
        assert_eq!(6, set.find(1));
        assert_eq!(6, set.find(2));
        assert_eq!(6, set.find(3));
        assert_eq!(4, set.find(4));
        assert_eq!(6, set.find(5));
        assert_eq!(6, set.find(6));

        set.union(0, 4);
        assert_eq!(7, set.find(0));
        assert_eq!(6, set.find(1));
        assert_eq!(6, set.find(2));
        assert_eq!(6, set.find(3));
        assert_eq!(7, set.find(4));
        assert_eq!(6, set.find(5));
        assert_eq!(6, set.find(6));
        assert_eq!(7, set.find(7));

        set.union(6, 7);
        assert_eq!(8, set.find(0));
        assert_eq!(8, set.find(1));
        assert_eq!(8, set.find(2));
        assert_eq!(8, set.find(3));
        assert_eq!(8, set.find(4));
        assert_eq!(8, set.find(5));
        assert_eq!(8, set.find(6));
        assert_eq!(8, set.find(7));
    }

    #[test]
    fn find_with_unions_all_at_once() {
        let mut set = LinkageUnionFind::with_len(5);

        set.union(1, 3);
        set.union(5, 2);
        set.union(0, 4);
        set.union(6, 7);

        // The set is now full, so everything should be in the same cluster.
        assert_eq!(8, set.find(0));
        assert_eq!(8, set.find(1));
        assert_eq!(8, set.find(2));
        assert_eq!(8, set.find(3));
        assert_eq!(8, set.find(4));
        assert_eq!(8, set.find(5));
        assert_eq!(8, set.find(6));
        assert_eq!(8, set.find(7));
    }

    #[test]
    fn union_is_idempotent() {
        let mut set = LinkageUnionFind::with_len(5);

        set.union(1, 3);
        set.union(5, 2);
        // `1` is already in the cluster `5`, so do a no-op union.
        set.union(5, 1);
        set.union(0, 4);
        set.union(6, 7);

        // The set is now full, so everything should be in the same cluster.
        assert_eq!(8, set.find(0));
        assert_eq!(8, set.find(1));
        assert_eq!(8, set.find(2));
        assert_eq!(8, set.find(3));
        assert_eq!(8, set.find(4));
        assert_eq!(8, set.find(5));
        assert_eq!(8, set.find(6));
        assert_eq!(8, set.find(7));

        // Union two clusters already in the same cluster when the set is full.
        set.union(1, 4);
        assert_eq!(8, set.find(0));
        assert_eq!(8, set.find(1));
        assert_eq!(8, set.find(2));
        assert_eq!(8, set.find(3));
        assert_eq!(8, set.find(4));
        assert_eq!(8, set.find(5));
        assert_eq!(8, set.find(6));
        assert_eq!(8, set.find(7));
    }

    #[test]
    fn relabel() {
        let mut den = Dendrogram::new(5);
        den.push(Step::new(1, 3, 0.01, 0));
        den.push(Step::new(1, 2, 0.02, 0));
        den.push(Step::new(0, 4, 0.015, 0));
        den.push(Step::new(1, 4, 0.03, 0));

        let mut set = LinkageUnionFind::new();
        set.relabel(&mut den, Method::Single);

        assert_eq!(den.steps(), &[
            Step::new(1, 3, 0.01, 2),
            Step::new(0, 4, 0.015, 2),
            Step::new(2, 5, 0.02, 3),
            Step::new(6, 7, 0.03, 5),
        ]);
    }
}
