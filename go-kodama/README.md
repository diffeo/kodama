go-kodama
=========
This package provides cgo bindings to the kodama hierarchical clustering
library.

This package is released under the MIT license.


### Documentation

[godoc.org/github.com/diffeo/kodama/go-kodama](http://godoc.org/github.com/diffeo/kodama/go-kodama)

The primary documentation for the Rust library, including a definition of the
syntax, can be found here:
https://docs.rs/kodama


### Install + Example

You'll need to [install Rust](https://www.rust-lang.org/downloads.html) (you'll
need at least Rust 1.19, which is the current stable release) and have a Go
compiler handy. To run tests for `go-kodama`, we'll need to compile the Rust
kodama library and then tell the Go compiler where to find it. These commands
should do it:

```
$ mkdir -p $GOPATH/src/github.com/diffeo
$ cd $GOPATH/src/github.com/diffeo
$ git clone git://github.com/diffeo/kodama
$ cd kodama/go-kodama
$ cargo build --release --manifest-path ../kodama-capi/Cargo.toml
$ export CGO_LDFLAGS="-L../kodama-capi/target/release"
$ export LD_LIBRARY_PATH="../kodama-capi/target/release"
```

Now you can run tests:

```
$ go test -v
=== RUN   TestLinkage64
--- PASS: TestLinkage64 (0.00s)
=== RUN   TestLinkage32
--- PASS: TestLinkage32 (0.00s)
PASS
ok      github.com/diffeo/kodama/go-kodama      0.003s
```

Or try compiling an example program:

```
$ go install github.com/diffeo/kodama/go-kodama/go-kodama-example
$ $GOPATH/bin/go-kodama-example
kodama.Step{Cluster1:2, Cluster2:4, Dissimilarity:3.1237967760688776, Size:2}
kodama.Step{Cluster1:5, Cluster2:6, Dissimilarity:5.757158112027513, Size:3}
kodama.Step{Cluster1:1, Cluster2:7, Dissimilarity:8.1392602685723, Size:4}
kodama.Step{Cluster1:3, Cluster2:8, Dissimilarity:12.483148228609206, Size:5}
kodama.Step{Cluster1:0, Cluster2:9, Dissimilarity:25.589444117482433, Size:6}
```

Note that, at least on Linux, the above setup will dynamically link the Rust
kodama library into the Go executable:

```
$ ldd /tmp/go/bin/go-kodama-example
...
        libkodama.so => ../kodama-capi/target/release/libkodama.so (0x00007f464cde6000)
...
```

It is possible to statically link kodama completely as well. For this, we need
to re-compile the kodama library into a static archive with musl, which will
statically link libc. This is easy to do if you have `rustup`, which permits
adding new targets. The new target can then be used with Cargo with the
`--target` flag. So to accomplish the above, we'll run a similar set of steps
for the setup:

```
$ mkdir -p $GOPATH/src/github.com/diffeo
$ cd $GOPATH/src/github.com/diffeo
$ git clone git://github.com/diffeo/kodama
$ cd kodama/go-kodama
```

And now we add musl and compile (note that different value of `CGO_LDFLAGS`!):

```
$ rustup target add x86_64-unknown-linux-musl
$ cargo build --release --manifest-path ../kodama-capi/Cargo.toml --target x86_64-unknown-linux-musl
$ export CGO_LDFLAGS="-L../kodama-capi/target/x86-64-unknown-linux-musl/release"
```

Now `go test` will work, and re-compiling `go-kodama-example` will also work,
but will no longer dynamically link with the kodama library:

```
$ go install github.com/diffeo/kodama/go-kodama/go-kodama-example
$ $GOPATH/bin/go-kodama-example
kodama.Step{Cluster1:2, Cluster2:4, Dissimilarity:3.1237967760688776, Size:2}
kodama.Step{Cluster1:5, Cluster2:6, Dissimilarity:5.757158112027513, Size:3}
kodama.Step{Cluster1:1, Cluster2:7, Dissimilarity:8.1392602685723, Size:4}
kodama.Step{Cluster1:3, Cluster2:8, Dissimilarity:12.483148228609206, Size:5}
kodama.Step{Cluster1:0, Cluster2:9, Dissimilarity:25.589444117482433, Size:6}
$ ldd $GOPATH/bin/go-kodama-example
        linux-vdso.so.1 (0x00007fff0cce3000)
        libpthread.so.0 => /usr/lib/libpthread.so.0 (0x00007f2237ee6000)
        libc.so.6 => /usr/lib/libc.so.6 (0x00007f2237b40000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f2238104000)
```

In fact, you can even go a step further and create a fully static executable:

```
$ go install -ldflags "-linkmode external -extldflags -static" github.com/diffeo/kodama/go-kodama/go-kodama-example
$ ldd $GOPATH/bin/go-kodama-example
        not a dynamic executable
```
