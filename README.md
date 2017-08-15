kodama
======
This crate provides a fast implementation of agglomerative
[hierarchical clustering](https://en.wikipedia.org/wiki/Hierarchical_clustering).

[![Linux build status](https://travis-ci.org/diffeo/kodama.svg?branch=master)](https://travis-ci.org/diffeo/kodama)
[![](https://img.shields.io/crates/v/kodama.svg)](https://crates.io/crates/kodama)

This library is released under the MIT license.

The ideas and implementation in this crate are heavily based on the work of
Daniel Müllner, and in particular, his 2011 paper,
[Modern hierarchical, agglomerative clustering algorithms](https://arxiv.org/pdf/1109.2378.pdf).
Parts of the implementation have also been inspired by his C++
library, [`fastcluster`](http://danifold.net/fastcluster.html).
Müllner's work, in turn, is based on the hierarchical clustering facilities
provided by MATLAB and
[SciPy](https://docs.scipy.org/doc/scipy/reference/generated/scipy.cluster.hierarchy.linkage.html).

The runtime performance of this library is on par with Müllner's `fastcluster`
implementation.

For a more detailed example of how to use hierarchical clustering, see the
[example in the API documentation](https://docs.rs/kodama/0.1.0/kodama/#example).

### Documentation

https://docs.rs/kodama

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
kodama = "0.1"
```

and this to your crate root:

```rust
extern crate kodama;
```
