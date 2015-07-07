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
