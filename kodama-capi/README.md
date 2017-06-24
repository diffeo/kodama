kodama-capi
===========
This crate exposes a C API for the kodama hierarchical clustering crate.

The header file (`includes/kodama.h`) serves as the primary API document for
the C API, but is not complete. The complete documentation can be found with
the Rust library.

This library is released under the MIT license.


### Usage

This library can be built like any other Rust library. By default, it should
produce both a shared object and a static library specific to your platform.
The resulting libraries expose a C ABI, and can be linked normally.


### Aborts

This library will abort your process if an unwinding panic is caught in the
Rust code. Generally, a panic occurs when there is a bug in the program (that
would be kodama in this case) or if allocation failed.

Note that this is orthogonal from other unchecked runtime errors. For example,
giving a NULL pointer to `kodama_dendrogram_steps` will result in attempting to
dereference a NULL pointer, which will probably result in a segmentation fault.
