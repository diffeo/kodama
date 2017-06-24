kodama
======
This crate provides a fast implementation of agglomerative
[hierarchical clustering](https://en.wikipedia.org/wiki/Hierarchical_clustering).

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
