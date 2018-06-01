// Package kodama provides cgo bindings to hierarchical clustering.
//
// The ideas and implementation in this crate are heavily based on the work of
// Daniel Müllner, and in particular, his 2011 paper, Modern hierarchical,
// agglomerative clustering algorithms. Parts of the implementation have also
// been inspired by his C++ library, fastcluster. Müllner's work, in turn,
// is based on the hierarchical clustering facilities provided by MATLAB and
// SciPy.
//
// The runtime performance of this library is on par with Müllner's
// fastcluster implementation.
//
// For more detailed information, see the documentation for the Rust library
// at https://docs.rs/kodama. Most or all of the things should translate
// straight-forwardly to these Go bindings.
package kodama

// #cgo LDFLAGS: -lkodama
// #include "kodama.h"
import "C"

import (
	"fmt"
	"math"
	"reflect"
	"runtime"
	"unsafe"
)

// Method indicates the update formula for computing dissimilarities between
// clusters.
//
// The method dictates how the dissimilarities are computed whenever a new
// cluster is formed. In particular, when clusters a and b are merged into a
// new cluster ab, then the pairwise dissimilarity between ab and every other
// cluster is computed using one of the variants of this type.
type Method int

// The available methods for computing linkage.
const (
	MethodSingle Method = iota
	MethodComplete
	MethodAverage
	MethodWeighted
	MethodWard
	MethodCentroid
	MethodMedian
)

// enum converts a Go method value into a C enum method value.
func (m Method) enum() C.kodama_method {
	switch m {
	case MethodSingle:
		return C.kodama_method_single
	case MethodComplete:
		return C.kodama_method_complete
	case MethodAverage:
		return C.kodama_method_average
	case MethodWeighted:
		return C.kodama_method_weighted
	case MethodWard:
		return C.kodama_method_ward
	case MethodCentroid:
		return C.kodama_method_centroid
	case MethodMedian:
		return C.kodama_method_median
	default:
		panic(fmt.Sprintf("unrecognized method: %v", m))
	}
}

// Dendrogram is a stepwise representation of a hierarchical clustering of
// N observations.
//
// A dendrogram consists of a series of N - 1 steps, where N is the number
// of observations that were clustered. Each step corresponds to the creation
// of a new cluster by merging exactly two previous clusters.
type Dendrogram struct {
	p *C.kodama_dendrogram
}

// newDendrogram creates a new dendrogram that wraps the C dendrogram.
//
// When the returned *Dendrogram is freed, then the C dendrogram should
// also be freed automatically.
func newDendrogram(cdend *C.kodama_dendrogram) *Dendrogram {
	dend := &Dendrogram{p: cdend}
	runtime.SetFinalizer(dend, func(dend *Dendrogram) {
		if dend.p != nil {
			C.kodama_dendrogram_free(dend.p)
			dend.p = nil
		}
	})
	return dend
}

// Len returns the number of steps in this dendrogram.
func (dend *Dendrogram) Len() int {
	return int(C.kodama_dendrogram_len(dend.p))
}

// Observations returns the number of observations in the data that is
// clustered by this dendrogram.
func (dend *Dendrogram) Observations() int {
	return int(C.kodama_dendrogram_observations(dend.p))
}

// Steps returns a slice of steps that make up the given dendrogram.
func (dend *Dendrogram) Steps() []Step {
	len := dend.Len()
	csteps := C.kodama_dendrogram_steps(dend.p)
	gosteps := (*[math.MaxInt32]C.kodama_step)(unsafe.Pointer(csteps))[:len:len]

	steps := make([]Step, len)
	for i, s := range gosteps {
		steps[i] = Step{
			Cluster1:      int(s.cluster1),
			Cluster2:      int(s.cluster2),
			Dissimilarity: float64(s.dissimilarity),
			Size:          int(s.size),
		}
	}
	return steps
}

// Step is a single merge step in a dendrogram.
//
// Each step corresponds to the creation of a new cluster by merging two
// previous clusters.
//
// By convention, the smaller cluster label is always assigned to the
// `cluster1` field.
type Step struct {
	// The label corresponding to the first cluster.
	Cluster1 int
	// The label corresponding to the second cluster.
	Cluster2 int
	// The dissimilarity between cluster1 and cluster2.
	Dissimilarity float64
	// The total number of observations in this merged cluster.
	Size int
}

// Linkage64 returns a hierarchical clustering of observations given their
// pairwise dissimilarities as double-precision floating point numbers.
//
// The pairwise dissimilarities must be provided as a *condensed pairwise
// dissimilarity matrix*, where only the values in the upper triangle are
// explicitly represented, not including the diagonal. As a result, the given
// matrix should have length observations-choose-2 (which is (observations *
// (observations - 1)) / 2) and only have values defined for pairs of (a, b)
// where a < b.
//
// The observations parameter is the total number of observations that are
// being clustered. Every pair of observations must have a finite non-NaN
// dissimilarity.
//
// The return value is a dendrogram. The dendrogram encodes a hierarchical
// clustering as a sequence of observations - 1 steps, where each step
// corresponds to the creation of a cluster by merging exactly two previous
// clusters. The very last cluster created contains all observations.
//
// If the length of the given matrix is not consistent with the number of
// observations, then this function will panic.
//
// The given matrix is never copied, but its values may be mutated during
// clustering.
func Linkage64(
	condensedDissimilarityMatrix []float64,
	observations int,
	method Method,
) *Dendrogram {
	expectedLen := (observations * (observations - 1)) / 2
	if len(condensedDissimilarityMatrix) != expectedLen {
		panic(fmt.Errorf(
			"expected dissimilarity matrix of length %d, but got %d",
			expectedLen, len(condensedDissimilarityMatrix)))
	}

	// Since we are reading this matrix (which is in Go memory) from
	// Rust, and since we are explicitly allowing zero-length slices, we
	// must ensure that we pass a non-null pointer to Rust. (If the Rust
	// bindings allowed a null pointer, then we'd wind up with UB.)
	if condensedDissimilarityMatrix == nil {
		condensedDissimilarityMatrix = []float64{}
	}
	header := (*reflect.SliceHeader)(unsafe.Pointer(&condensedDissimilarityMatrix))
	cmat := (*C.double)(unsafe.Pointer(header.Data))
	return newDendrogram(C.kodama_linkage_double(cmat, C.size_t(observations), method.enum()))
}

// Linkage32 returns a hierarchical clustering of observations given their
// pairwise dissimilarities as single-precision floating point numbers.
//
// The pairwise dissimilarities must be provided as a *condensed pairwise
// dissimilarity matrix*, where only the values in the upper triangle are
// explicitly represented, not including the diagonal. As a result, the given
// matrix should have length observations-choose-2 (which is (observations *
// (observations - 1)) / 2) and only have values defined for pairs of (a, b)
// where a < b.
//
// The observations parameter is the total number of observations that are
// being clustered. Every pair of observations must have a finite non-NaN
// dissimilarity.
//
// The return value is a dendrogram. The dendrogram encodes a hierarchical
// clustering as a sequence of observations - 1 steps, where each step
// corresponds to the creation of a cluster by merging exactly two previous
// clusters. The very last cluster created contains all observations.
//
// If the length of the given matrix is not consistent with the number of
// observations, then this function will panic.
//
// The given matrix is never copied, but its values may be mutated during
// clustering.
func Linkage32(
	condensedDissimilarityMatrix []float32,
	observations int,
	method Method,
) *Dendrogram {
	expectedLen := (observations * (observations - 1)) / 2
	if len(condensedDissimilarityMatrix) != expectedLen {
		panic(fmt.Errorf(
			"expected dissimilarity matrix of length %d, but got %d",
			expectedLen, len(condensedDissimilarityMatrix)))
	}

	// Since we are reading this matrix (which is in Go memory) from
	// Rust, and since we are explicitly allowing zero-length slices, we
	// must ensure that we pass a non-null pointer to Rust. (If the Rust
	// bindings allowed a null pointer, then we'd wind up with UB.)
	if condensedDissimilarityMatrix == nil {
		condensedDissimilarityMatrix = []float32{}
	}
	header := (*reflect.SliceHeader)(unsafe.Pointer(&condensedDissimilarityMatrix))
	cmat := (*C.float)(unsafe.Pointer(header.Data))
	return newDendrogram(C.kodama_linkage_float(cmat, C.size_t(observations), method.enum()))
}
