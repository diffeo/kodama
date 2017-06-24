#ifndef _KODAMA_H
#define _KODAMA_H

#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * A stepwise dendrogram.
 *
 * A dendrogram consists of a series of `N - 1` steps, where `N` is the number
 * of observations that were clustered.
 *
 * Users of this library responsible for freeing the resources associated with
 * a dendrogram using `kodama_dendrogram_free`.
 */
typedef struct kodama_dendrogram kodama_dendrogram;

/**
 * A single step in a dendrogram.
 *
 * Each step corresponds to the creation of a new cluster by merging two
 * previous clusters.
 *
 * By convention, the smaller cluster label is always assigned to the
 * `cluster1` field.
 */
typedef struct kodama_step {
    /* The label corresponding to the first cluster. */
    size_t cluster1;
    /* The label corresponding to the second cluster. */
    size_t cluster2;
    /* The dissimilarity between cluster1 and cluster2. */
    double dissimilarity;
    /* The total number of observations in this merged cluster. */
    size_t size;
} kodama_step;

/**
 * A method for computing dissimilarities between clusters.
 *
 * The method dictates how the dissimilarities are computed whenever a new
 * cluster is formed. In particular, when clusters `a` and `b` are merged into
 * a new cluster `ab`, then the pairwise dissimilarity between `ab` and every
 * other cluster is computed using one of the variants of this type.
 */
typedef enum kodama_method {
    kodama_method_single,
    kodama_method_complete,
    kodama_method_average,
    kodama_method_weighted,
    kodama_method_ward,
    kodama_method_centroid,
    kodama_method_median,
} kodama_method;

/**
 * Return a hierarchical clustering of observations given their pairwise
 * dissimilarities as double-precision floating point numbers.
 *
 * The pairwise dissimilarities must be provided as a *condensed pairwise
 * dissimilarity matrix*, where only the values in the upper triangle are
 * explicitly represented, not including the diagonal. As a result, the given
 * matrix should have length `observations-choose-2` (which is
 * `(observations * (observations - 1)) / 2`) and only have values defined for
 * pairs of `(a, b)` where `a < b`.
 *
 * `observations` is the total number of observations that are being clustered.
 * Every pair of observations must have a finite non-NaN dissimilarity.
 *
 * The return value is a dendrogram that the caller is responsible for freeing.
 * The dendrogram encodes a hierarchical clustering as a sequence of
 * `observations - 1` steps, where each step corresponds to the creation of
 * a cluster by merging exactly two previous clusters. The very last cluster
 * created contains all observations.
 *
 * It is an unchecked runtime error to provide a matrix with a length that is
 * consistent with the number of observations.
 *
 * The given matrix is never copied, but its values may be mutated during
 * clustering.
 */
kodama_dendrogram *kodama_linkage_double(
    double *condensed_dissimilarity_matrix,
    size_t observations,
    kodama_method method);

/**
 * Return a hierarchical clustering of observations given their pairwise
 * dissimilarities as single-precision floating point numbers.
 *
 * The pairwise dissimilarities must be provided as a *condensed pairwise
 * dissimilarity matrix*, where only the values in the upper triangle are
 * explicitly represented, not including the diagonal. As a result, the given
 * matrix should have length `observations-choose-2` (which is
 * `(observations * (observations - 1)) / 2`) and only have values defined for
 * pairs of `(a, b)` where `a < b`.
 *
 * `observations` is the total number of observations that are being clustered.
 * Every pair of observations must have a finite non-NaN dissimilarity.
 *
 * The return value is a dendrogram that the caller is responsible for freeing.
 * The dendrogram encodes a hierarchical clustering as a sequence of
 * `observations - 1` steps, where each step corresponds to the creation of
 * a cluster by merging exactly two previous clusters. The very last cluster
 * created contains all observations.
 *
 * It is an unchecked runtime error to provide a matrix with a length that is
 * consistent with the number of observations.
 *
 * The given matrix is never copied, but its values may be mutated during
 * clustering.
 */
kodama_dendrogram *kodama_linkage_float(
    float *condensed_dissimilarity_matrix,
    size_t observations,
    kodama_method method);

/**
 * Returns the total number of steps in the given dendrogram.
 */
size_t kodama_dendrogram_len(const kodama_dendrogram *dend);

/**
 * Returns the total number of observations that are clustered in this
 * dendrogram.
 *
 * This is always equivalent to the number of observations provided in the
 * linkage clustering functions.
 */
size_t kodama_dendrogram_observations(const kodama_dendrogram *dend);

/**
 * Return an array of steps that make up the given dendrogram.
 *
 * The array returned should NOT be freed by the caller. Instead, the resources
 * consumed by the array should be freed by freeing the entire dendrogram.
 */
kodama_step *kodama_dendrogram_steps(const kodama_dendrogram *dend);

/**
 * Free the resources associated with the given dendrogram.
 *
 * A dendrogram cannot be used after it has been freed.
 */
void kodama_dendrogram_free(kodama_dendrogram *dend);

#ifdef __cplusplus
}
#endif

#endif
