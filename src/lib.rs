//! # Rust binding for HdrHistogram_c
//!
//! This crate implements bindings for
//! [HdrHistogram_c](https://github.com/hdrhistogram/HdrHistogram_c), a flexible library for
//! recording histograms without having to know very much about the data being histogrammed.
//!
//! The top-level type is `Histogram`.
//!
//! # Example
//!
//! This sets up a histogram to record values in the range 1..1000_000 with 2 significant figures of
//! precision. It then records 
//!
//! ```
//! # use hdrhistogram::Histogram;
//! let mut h = Histogram::init(1, 1000000, 2).unwrap();
//! 
//! h.record_value(1);
//! h.record_value(10);
//! h.record_value(100, 40);
//!
//! assert_eq!(h.total_count(), 42);
//! assert_eq!(h.min(), 1);
//! assert_eq!(h.max(), 100);
//! ```

mod ffi;
mod dblffi;

pub use ffi::{Histogram, HistogramBucketCfg,
              LinearIter, LogIter, PercentileIter, RecordedIter,
              CountIterItem, PercentileIterItem };

pub use dblffi::{F64Histogram};

#[cfg(test)]
mod test;

#[cfg(test)]
mod dbltest;
