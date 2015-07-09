//! # Rust binding for HdrHistogram_c
//!
//! This crate implements bindings for
//! [HdrHistogram_c](https://github.com/hdrhistogram/HdrHistogram_c), a flexible library for
//! recording histograms without having to know very much about the data being histogrammed.
//!
//! The top-level type is `Histogram`.
//!
//! # Example
//! ```
//! # use hdrhistogram::Histogram;
//! let mut h = Histogram::init(1, 1000000, 2).unwrap();
//! 
//! h.record_value(1);
//! h.record_value(10);
//!
//! assert_eq!(h.total_count(), 2);
//! assert_eq!(h.min(), 1);
//! assert_eq!(h.max(), 10);
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
