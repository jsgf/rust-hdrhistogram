mod ffi;

pub use ffi::{Histogram, HistogramBucketCfg,
              LinearIter, LogIter, PercentileIter, RecordedIter,
              CountIterItem, PercentileIterItem };

#[cfg(test)]
mod test;
