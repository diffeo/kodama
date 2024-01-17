package main

import (
	"fmt"
	// currently this example cannot be compiled due to a missing library
	// kodama "github.com/diffeo/kodama/go-kodama"
)

func main() {
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
	fmt.Printf("%#v\n %v", maCondensedMatrix64, maObservations);
	/*
 	dend := kodama.Linkage64(maCondensedMatrix64, maObservations, kodama.MethodAverage)
	for _, step := range dend.Steps() {
		fmt.Printf("%#v\n", step)
	}*/
}
