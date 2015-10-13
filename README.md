Rust binding to HdrHistogram_c library
======================================

[![Build Status](https://travis-ci.org/jsgf/rust-hdrhistogram.svg?branch=master)](https://travis-ci.org/jsgf/rust-hdrhistogram)

This is a thin binding to the [HdrHistogram_c](https://github.com/HdrHistogram/HdrHistogram_c)
library, itself a port of [HdrHistogram](http://hdrhistogram.org/). Aside from the normal Rust
safety features, the main embellishment is an implementation of the Iterator trait for the various
ways to iterate the histogram.

It also uses `u64` instead of a signed type for values, as the library does not allow values to be
less than 1. However it also means that any value greater than 2^63 will be treated as negative and
rejected.

I've re-implemented the test suite in Rust (cargo test) to exercise the API, and it all passes.

TODO:
 * Finish basic API
 * Complete iterator items
 * anything missing

Use
---

This is on crates.io, so using it is just a matter of adding this to your Cargo.toml:

```
[dependencies]
hdrhistogram = "*"
```

The API is not at all stable right now.

Documentation
-------------

Docs are hosted [here](https://jsgf.github.io/hdrhistogram/).

Jeremy Fitzhardinge <jeremy@goop.org>
