package kodama

import (
	"math"
	"testing"
)

// The number of observations in our tiny test data set.
const maObservations = 6

// Distance in miles between 6 municipalities in central Massachusetts.
// Distances were computed using the Haversine formula.
var maCondensedMatrix64 = []float64{
	28.798738047815913, /* fitchburg, framingham */
	20.776023574084647, /* fitchburg, marlborough */
	30.846454181742043, /* fitchburg, northbridge */
	23.852344515986452, /* fitchburg, southborough */
	23.67366026778309,  /* fitchburg, westborough */
	8.3414966246663,    /* framingham, marlborough */
	14.849621987949059, /* framingham, northbridge */
	5.829368809982563,  /* framingham, southborough */
	10.246915371068036, /* framingham, westborough */
	14.325455610728019, /* marlborough, northbridge */
	3.1237967760688776, /* marlborough, southborough */
	6.205979766034621,  /* marlborough, westborough */
	12.424204118142217, /* northbridge, southborough */
	8.333311197617531,  /* northbridge, westborough */
	5.308336458020405,  /* southborough, westborough */
}

// The expected stepwise dendrogram from clustering the above dissimilarities
// using average linkage.
var maSteps = []Step{
	{2, 4, 3.1237967760688776, 2},
	{5, 6, 5.757158112027513, 3},
	{1, 7, 8.1392602685723, 4},
	{3, 8, 12.483148228609206, 5},
	{0, 9, 25.589444117482433, 6},
}

func TestLinkage64(t *testing.T) {
	dis := make([]float64, len(maCondensedMatrix64))
	copy(dis, maCondensedMatrix64)

	dend := Linkage64(dis, maObservations, MethodAverage)
	if dend.Len() != maObservations-1 {
		t.Fatalf("expected %d steps, but got %d\n", maObservations-1, dend.Len())
	}
	steps := dend.Steps()
	for i := range steps {
		assertStepApproxEq(t, i, steps[i], maSteps[i])
	}
}

func TestLinkage32(t *testing.T) {
	dis := make([]float32, len(maCondensedMatrix64))
	for i, x := range maCondensedMatrix64 {
		dis[i] = float32(x)
	}

	dend := Linkage32(dis, maObservations, MethodAverage)
	if dend.Len() != maObservations-1 {
		t.Fatalf("expected %d steps, but got %d\n", maObservations-1, dend.Len())
	}
	steps := dend.Steps()
	for i := range steps {
		assertStepApproxEq(t, i, steps[i], maSteps[i])
	}
}

func TestLinkage64Empty(t *testing.T) {
	// nil slice
	var dis []float64
	if dend := Linkage64(dis, 0, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}

	// empty slice
	dis = []float64{}
	if dend := Linkage64(dis, 0, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}

	// empty slice, but 1 observation
	// (1 observation has an empty dissimilarity matrix)
	if dend := Linkage64(dis, 1, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}
}

func TestLinkage64EmptySteps(t *testing.T) {
	dis := []float64{}
	dend := Linkage64(dis, 0, MethodAverage)
	if steps := dend.Steps(); len(steps) != 0 {
		t.Fatalf("expected zero steps, but got %d steps\n", len(steps))
	}

	dend = Linkage64(dis, 1, MethodAverage)
	if steps := dend.Steps(); len(steps) != 0 {
		t.Fatalf("expected zero steps, but got %d steps\n", len(steps))
	}
}

func TestLinkage32Empty(t *testing.T) {
	// nil slice
	var dis []float32
	if dend := Linkage32(dis, 0, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}

	// empty slice
	dis = []float32{}
	if dend := Linkage32(dis, 0, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}

	// empty slice, but 1 observation
	// (1 observation has an empty dissimilarity matrix)
	if dend := Linkage32(dis, 1, MethodAverage); dend.Len() != 0 {
		t.Fatalf("expected empty dendrogram, but got one of length %d\n", dend.Len())
	}
}

func TestLinkage32EmptySteps(t *testing.T) {
	dis := []float32{}
	dend := Linkage32(dis, 0, MethodAverage)
	if steps := dend.Steps(); len(steps) != 0 {
		t.Fatalf("expected zero steps, but got %d steps\n", len(steps))
	}

	dend = Linkage32(dis, 1, MethodAverage)
	if steps := dend.Steps(); len(steps) != 0 {
		t.Fatalf("expected zero steps, but got %d steps\n", len(steps))
	}
}

func assertStepApproxEq(t *testing.T, stepIndex int, got, expected Step) {
	eps := 0.000001
	if math.Abs(got.Dissimilarity-expected.Dissimilarity) > eps {
		t.Fatalf(
			"step %d dissimilarities not equal, got %f but expected %f\n",
			stepIndex, got.Dissimilarity, expected.Dissimilarity)
	}
	if got.Cluster1 != expected.Cluster1 {
		t.Fatalf(
			"step %d cluster1 label not equal, got %d but expected %d\n",
			stepIndex, got.Cluster1, expected.Cluster1)
	}
	if got.Cluster2 != expected.Cluster2 {
		t.Fatalf(
			"step %d cluster2 label not equal, got %d but expected %d\n",
			stepIndex, got.Cluster2, expected.Cluster2)
	}
	if got.Size != expected.Size {
		t.Fatalf(
			"step %d size not equal, got %d but expected %d\n",
			stepIndex, got.Size, expected.Size)
	}
}
