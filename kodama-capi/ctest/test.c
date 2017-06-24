#include <assert.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "kodama.h"

#ifndef DEBUG
  #define DEBUG false
#endif

/**
 * The number of observations in our tiny data set.
 */
size_t MA_OBSERVATIONS = 6;

/**
 * Distance in miles between 6 municipalities in central Massachusetts.
 * Distances were computed using the Haversine formula.
 */
double MA_CONDENSED_MATRIX_DOUBLE[] = {
    28.798738047815913,  /* fitchburg, framingham */
    20.776023574084647,  /* fitchburg, marlborough */
    30.846454181742043,  /* fitchburg, northbridge */
    23.852344515986452,  /* fitchburg, southborough */
    23.67366026778309,   /* fitchburg, westborough */
    8.3414966246663,     /* framingham, marlborough */
    14.849621987949059,  /* framingham, northbridge */
    5.829368809982563,   /* framingham, southborough */
    10.246915371068036,  /* framingham, westborough */
    14.325455610728019,  /* marlborough, northbridge */
    3.1237967760688776,  /* marlborough, southborough */
    6.205979766034621,   /* marlborough, westborough */
    12.424204118142217,  /* northbridge, southborough */
    8.333311197617531,   /* northbridge, westborough */
    5.308336458020405    /* southborough, westborough */
};

/**
 * Same as MA_CONDENSED_MATRIX_DOUBLE, but single precision.
 */
float MA_CONDENSED_MATRIX_FLOAT[] = {
    28.798738047815913,  /* fitchburg, framingham */
    20.776023574084647,  /* fitchburg, marlborough */
    30.846454181742043,  /* fitchburg, northbridge */
    23.852344515986452,  /* fitchburg, southborough */
    23.67366026778309,   /* fitchburg, westborough */
    8.3414966246663,     /* framingham, marlborough */
    14.849621987949059,  /* framingham, northbridge */
    5.829368809982563,   /* framingham, southborough */
    10.246915371068036,  /* framingham, westborough */
    14.325455610728019,  /* marlborough, northbridge */
    3.1237967760688776,  /* marlborough, southborough */
    6.205979766034621,   /* marlborough, westborough */
    12.424204118142217,  /* northbridge, southborough */
    8.333311197617531,   /* northbridge, westborough */
    5.308336458020405    /* southborough, westborough */
};

/**
 * The expected stepwise dendrogram from clustering the above dissimilarities
 * using average linkage.
 */
kodama_step MA_STEPS[] = {
    {2, 4, 3.1237967760688776, 2},
    {5, 6, 5.757158112027513, 3},
    {1, 7, 8.1392602685723, 4},
    {3, 8, 12.483148228609206, 5},
    {0, 9, 25.589444117482433, 6}
};

bool eq_with_epsilon(double x, double y, double epsilon) {
    return fabs(x - y) <= epsilon;
}

bool assert_step_approx_eq(
    const char *name,
    int i,
    kodama_step *got,
    kodama_step *expected
) {
    double eps = 0.000001;
    bool yes = eq_with_epsilon(
        got->dissimilarity, expected->dissimilarity, eps);
    if (!yes && DEBUG) {
        fprintf(stderr,
                "[%s] step %d dissimilarities not equal, "
                "got %0.15f but expected %0.15f\n",
                name, i, got->dissimilarity, expected->dissimilarity);
    }
    if (got->cluster1 != expected->cluster1) {
        yes = false;
        fprintf(stderr,
                "[%s] step %d cluster1 label not equal, "
                "got %zu but expected %zu\n",
                name, i, got->cluster1, expected->cluster1);
    }
    if (got->cluster2 != expected->cluster2) {
        yes = false;
        fprintf(stderr,
                "[%s] step %d cluster2 label not equal, "
                "got %zu but expected %zu\n",
                name, i, got->cluster2, expected->cluster2);
    }
    if (got->size != expected->size) {
        yes = false;
        fprintf(stderr,
                "[%s] step %d size not equal, "
                "got %zu but expected %zu\n",
                name, i, got->size, expected->size);
    }
    return yes;
}

double *ma_condensed_matrix_double() {
    double *dis = malloc(15 * sizeof(*dis));
    assert(dis != NULL);
    memcpy(dis, MA_CONDENSED_MATRIX_DOUBLE, 15 * sizeof(*dis));
    return dis;
}

float *ma_condensed_matrix_float() {
    float *dis = malloc(15 * sizeof(*dis));
    assert(dis != NULL);
    memcpy(dis, MA_CONDENSED_MATRIX_FLOAT, 15 * sizeof(*dis));
    return dis;
}

bool test_linkage_double() {
    bool passed = true;

    double *matrix = ma_condensed_matrix_double();
    kodama_dendrogram *dend = kodama_linkage_double(
        matrix, MA_OBSERVATIONS, kodama_method_average);

    size_t got_len = kodama_dendrogram_len(dend);
    if (got_len != MA_OBSERVATIONS - 1) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_linkage_double] expected %zu steps, but got %zu\n",
                    MA_OBSERVATIONS - 1, got_len);
        }
        passed = false;
    }

    kodama_step *steps = kodama_dendrogram_steps(dend);
    for (size_t i = 0; i < got_len; i++)
        passed = passed && assert_step_approx_eq(
            "test_linkage_double", i, &steps[i], &MA_STEPS[i]);

    kodama_dendrogram_free(dend);
    free(matrix);
    return passed;
}

bool test_linkage_float() {
    bool passed = true;

    float *matrix = ma_condensed_matrix_float();
    kodama_dendrogram *dend = kodama_linkage_float(
        matrix, MA_OBSERVATIONS, kodama_method_average);

    size_t got_len = kodama_dendrogram_len(dend);
    if (got_len != MA_OBSERVATIONS - 1) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_linkage_float] expected %zu steps, but got %zu\n",
                    MA_OBSERVATIONS - 1, got_len);
        }
        passed = false;
    }

    kodama_step *steps = kodama_dendrogram_steps(dend);
    for (size_t i = 0; i < got_len; i++)
        passed = passed && assert_step_approx_eq(
            "test_linkage_float", i, &steps[i], &MA_STEPS[i]);

    kodama_dendrogram_free(dend);
    free(matrix);
    return passed;
}

void run_test(bool (test)(), const char *name, bool *passed) {
    if (!test()) {
        *passed = false;
        fprintf(stderr, "FAILED: %s\n", name);
    } else {
        fprintf(stderr, "PASSED: %s\n", name);
    }
}

int main() {
    bool passed = true;

    run_test(test_linkage_double, "test_linkage_double", &passed);
    run_test(test_linkage_float, "test_linkage_float", &passed);

    if (!passed) {
        exit(1);
    }
    return 0;
}
