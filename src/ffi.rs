extern crate libc;

use self::libc::{c_double, c_int, c_void, c_char, size_t};
use std::str;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::fmt;
use std::error::Error;
use std::mem;
use std::ptr;
use std::result;

pub use super::Result;

type int64_t = i64;
type int32_t = i32;

#[cfg(test)]
pub mod test;

#[repr(C)]
#[derive(Debug)]
struct hdr_histogram {
    lowest_trackable_value: int64_t,
    highest_trackable_value: int64_t,
    unit_magnitude: int32_t,
    significant_figures: int32_t,
    sub_bucket_half_count_magnitude: int32_t,
    sub_bucket_half_count: int32_t,
    sub_bucket_mask: int64_t,
    sub_bucket_count: int32_t,
    bucket_count: int32_t,
    min_value: int64_t,
    max_value: int64_t,
    normalizing_index_offset: int32_t,
    conversion_ratio: c_double,
    counts_len: int32_t,
    total_count: int64_t,
    counts: [int64_t; 0],
}

#[repr(C)]
#[derive(Default,Debug,Copy,Clone)]
struct hdr_histogram_bucket_config {
    lowest_trackable_value: int64_t,
    highest_trackable_value: int64_t,
    unit_magnitude: int64_t,
    significant_figures: int64_t,
    sub_bucket_half_count_magnitude: int32_t,
    sub_bucket_half_count: int32_t,
    sub_bucket_mask: int64_t,
    sub_bucket_count: int32_t,
    bucket_count: int32_t,
    counts_len: int32_t,
}
        

#[repr(C)]
struct hdr_iter_percentiles {
    seen_last_value: bool,
    ticks_per_half_histance: int32_t,
    percentile_to_iterate_to: c_double,
    percentile: c_double,
}

#[repr(C)]
struct hdr_iter_recorded {
    count_added_in_this_iteration_step: int64_t,
}

#[repr(C)]
struct hdr_iter_linear {
    value_units_per_bucket: int64_t,
    count_added_in_this_iteration_step: int64_t,
    next_value_reporting_level: int64_t,
    next_value_reporting_level_lowest_equivalent: int64_t,
}

#[repr(C)]
struct hdr_iter_log {
    log_base: c_double,
    count_added_in_this_iteration_step: int64_t,
    next_value_reporting_level: int64_t,
    next_value_reporting_level_lowest_equivalent: int64_t,
}

#[repr(C)]
#[derive(Debug)]
struct hdr_iter {
    h: *const hdr_histogram,

    /** raw index into the counts array */
    counts_index: int32_t,
    /** value directly from array for the current counts_index */
    count: int64_t,
    /** sum of all of the counts up to and including the count at this index */
    cumulative_count: int64_t,
    /** The current value based on counts_index */
    value: int64_t,
    highest_equivalent_value: int64_t,
    lowest_equivalent_value: int64_t,
    median_equivalent_value: int64_t,
    value_iterated_from: int64_t,
    value_iterated_to: int64_t,

    union: [int64_t; 4],        // hdr_iter_log/linear

    _next_fp: *const c_void,    // function pointer
}

impl Default for hdr_iter {
    fn default() -> Self { unsafe { mem::zeroed() } }
}

#[allow(dead_code)]
extern {
    fn hdr_init(lowest_trackable_value: int64_t, highest_trackable_value: int64_t,
                significant_figures: c_int, res: *mut *mut hdr_histogram) -> c_int;
    fn hdr_reset(h: *mut hdr_histogram);
    fn hdr_get_memory_size(h: *const hdr_histogram) -> size_t;

    fn hdr_record_value(h: *mut hdr_histogram, value: int64_t) -> bool;
    fn hdr_record_values(h: *mut hdr_histogram, value: int64_t, count: int64_t) -> bool;
    fn hdr_record_corrected_value(h: *mut hdr_histogram, value: int64_t, expected_interval: int64_t) -> bool;
    fn hdr_record_corrected_values(h: *mut hdr_histogram, value: int64_t, count: int64_t,
                                   expected_interval: int64_t) -> bool;

    fn hdr_add(h: *mut hdr_histogram, from: *const hdr_histogram) -> int64_t;
    fn hdr_add_while_correcting_for_coordinated_omission(h: *mut hdr_histogram, from: *const hdr_histogram,
                                                         expected_interval: int64_t) -> int64_t;

    fn hdr_min(h: *const hdr_histogram) -> int64_t;
    fn hdr_max(h: *const hdr_histogram) -> int64_t;

    fn hdr_value_at_percentile(h: *const hdr_histogram, percentile: c_double) -> int64_t;

    fn hdr_stddev(h: *const hdr_histogram) -> c_double;
    fn hdr_mean(h: *const hdr_histogram) -> c_double;

    fn hdr_values_are_equivalent(h: *const hdr_histogram, a: int64_t, b: int64_t) -> bool;
    fn hdr_lowest_equivalent_value(h: *const hdr_histogram, value: int64_t) -> int64_t;
    fn hdr_count_at_value(h: *const hdr_histogram, value: int64_t) -> int64_t;
    fn hdr_count_at_index(h: *const hdr_histogram, index: int32_t) -> int64_t;
    fn hdr_value_at_index(h: *const hdr_histogram, index: int32_t) -> int64_t;
    
    fn hdr_calculate_bucket_config(lowest_trackable_value: int64_t, highest_trackable_value: int64_t,
                                   significant_figures: c_int, cfg: *mut hdr_histogram_bucket_config) -> c_int;
    fn hdr_init_preallocated(h: *mut hdr_histogram, cfg: *const hdr_histogram_bucket_config);

    fn hdr_shift_values_left(h: *mut hdr_histogram, binary_orders_of_magnitude: int32_t) -> bool;
    fn hdr_shift_values_right(h: *mut hdr_histogram, shift: int32_t) -> bool;
    fn hdr_size_of_equivalent_value_range(h: *const hdr_histogram, value: int64_t) -> int64_t;
    fn hdr_next_non_equivalent_value(h: *const hdr_histogram, value: int64_t) -> int64_t;
    fn hdr_median_equivalent_value(h: *const hdr_histogram, value: int64_t) -> int64_t;

    fn hdr_iter_init(iter: *mut hdr_iter, h: *const hdr_histogram);
    fn hdr_iter_percentile_init(iter: *mut hdr_iter, h: *const hdr_histogram, ticks_per_half_distance: int32_t);
    fn hdr_iter_recorded_init(iter: *mut hdr_iter, h: *const hdr_histogram);
    fn hdr_iter_linear_init(iter: *mut hdr_iter, h: *const hdr_histogram, value_units_per_bucket: int64_t);
    fn hdr_iter_log_init(iter: *mut hdr_iter, h: *const hdr_histogram, value_units_per_bucket: int64_t, log_base: c_double);

    fn hdr_iter_next(iter: *mut hdr_iter) -> bool;

    // hdr_histogram_log

    fn hdr_log_encode(histogram: *const hdr_histogram, encoded_histogram: *mut *mut c_char) -> c_int;
    fn hdr_log_decode(histogram: *mut *mut hdr_histogram, base64_histogram: *const c_char, base64_len: size_t) -> c_int;

    fn hdr_strerror(errnum: c_int) -> *const c_char;
}

/// Catch-all error return.
///
/// Something went wrong.
#[derive(Copy,Clone,Debug)]
pub struct HistogramErr(&'static str);

impl fmt::Display for HistogramErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let &HistogramErr(msg) = self;
        msg.fmt(fmt)
    }
}

impl Error for HistogramErr {
    fn description(&self) -> &str { let &HistogramErr(msg) = self; msg }
}

/// Instance of a Histogram.
pub struct Histogram {
    histo: *mut hdr_histogram,
    owned: bool,
}

unsafe impl Send for Histogram {}

impl Histogram {
    /// Create a new Histogram instance. `lowest_trackable_value`..`highest_trackable_value` defines
    /// the range of values in the `Histogram`. `lowest_trackable_value` must be >= 1.
    /// `significant_figures` defines the precision, and must be in the range 1..5.
    ///
    /// `HistogramErr` is the catch-all failure case. It doesn't report much detail because the
    /// underlying library doesn't.
    ///
    /// # Example
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// let mut h = Histogram::init(1, 100000, 2).unwrap();
    /// h.record_value(10);  // record a single count of '10'
    /// ```
    pub fn init(lowest_trackable_value: u64, highest_trackable_value: u64, significant_figures: u32) -> Result<Histogram> {
        let mut histo : *mut hdr_histogram = ptr::null_mut();
        let r = unsafe {
            hdr_init(lowest_trackable_value as int64_t, highest_trackable_value as int64_t,
                     significant_figures as int32_t, &mut histo)
        };

        if r != 0 || histo.is_null() {
            Err(HistogramErr("Histogram init failed"))
        } else {
            Ok(Histogram { histo: histo, owned: true })
        }
    }

    /// Zero all histogram state in place.
    pub fn reset(&mut self) { unsafe { hdr_reset(self.histo) } }

    /// Record a specific value. Returns true if successful.
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// # let mut h = Histogram::init(1, 10, 1).unwrap();
    /// h.record_value(5);
    /// assert_eq!(h.total_count(), 1);
    /// ```
    #[inline]
    pub fn record_value(&mut self, value: u64) -> bool {
        unsafe { hdr_record_value(self.histo, value as int64_t) }
    }

    /// Record multiple counts of a specific value. Returns true if successful.
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// # let mut h = Histogram::init(1, 10, 1).unwrap();
    /// h.record_values(5, 10);
    /// assert_eq!(h.total_count(), 10);
    /// ```
    #[inline]
    pub fn record_values(&mut self, value: u64, count: u64) -> bool {
        unsafe { hdr_record_values(self.histo, value as int64_t, count as int64_t) }
    }

    /// Record a value, correcting for coordinated omission. This should be used when accumulating
    /// latency measurements taked on a regular timebase (expected_interval). Any latency that's
    /// well above that interval implies some kind of outage in which sampled were lost. This
    /// corrects for those lost samples to preserve the integrity of the overall statistics.
    #[inline]
    pub fn record_corrected_value(&mut self, value: u64, expected_interval: u64) -> bool {
        unsafe { hdr_record_corrected_value(self.histo, value as int64_t, expected_interval as int64_t) }
    }

    /// As with `record_corrected_value()` but multiple counts of the value.
    #[inline]
    pub fn record_corrected_values(&mut self, value: u64, count: u64, expected_interval: u64) -> bool {
        unsafe { hdr_record_corrected_values(self.histo, value as int64_t, count as int64_t, expected_interval as int64_t) }
    }

    /// Sum two histograms, modifying `self` in place. Returns the number of samples dropped;
    /// samples will be dropped if they're out of the range `lowest_trackable_value
    /// .. highest_trackable_value`.
    pub fn add(&mut self, other: &Histogram) -> u64 {
        unsafe { hdr_add(self.histo, other.histo) as u64 }
    }

    /// As with `add` but corrects of potential lost samples while doing periodic latency
    /// measurements, as in `record_corrected_value`.  Only one correction should be applied.
    pub fn add_while_correcting_for_coordinated_omission(&mut self, other: &Histogram, expected_interval: u64) -> u64 {
        unsafe {
            hdr_add_while_correcting_for_coordinated_omission(self.histo, other.histo, expected_interval as int64_t) as u64
        }
    }

    /// Total of all counters
    pub fn total_count(&self) -> u64 { unsafe { (*self.histo).total_count as u64 } }

    /// Smallest recorded value.
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// # let mut h = Histogram::init(1, 10, 1).unwrap();
    /// h.record_value(1);
    /// h.record_value(5);
    /// assert_eq!(h.min(), 1);
    /// ```
    pub fn min(&self) -> u64 {
        unsafe { hdr_min(self.histo) as u64 }
    }

    /// Largest recorded value.
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// # let mut h = Histogram::init(1, 10, 1).unwrap();
    /// h.record_value(1);
    /// h.record_value(5);
    /// assert_eq!(h.max(), 5);
    /// ```
    pub fn max(&self) -> u64 {
        unsafe { hdr_max(self.histo) as u64 }
    }

    /// Value at a particular percentile (0-100).
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// # let mut h = Histogram::init(1, 100, 2).unwrap();
    /// h.record_values(20, 10);
    /// h.record_value(40);
    /// assert_eq!(h.value_at_percentile(50.0), 20);
    /// assert_eq!(h.value_at_percentile(99.0), 40);
    pub fn value_at_percentile(&self, percentile: f64) -> u64 {
        unsafe { hdr_value_at_percentile(self.histo, percentile) as u64 }
    }

    /// Standard deviation of values.
    pub fn stddev(&self) -> f64 {
        unsafe { hdr_stddev(self.histo) }
    }

    /// Mean of values.
    pub fn mean(&self) -> f64 {
        unsafe { hdr_mean(self.histo) }
    }

    /// Returns true if two values are the same according to the lowest, highest and significant
    /// figures parameters.
    pub fn values_are_equivalent(&self, a: u64, b: u64) -> bool {
        unsafe { hdr_values_are_equivalent(self.histo, a as int64_t, b as int64_t) }
    }

    /// Lowest value equivalent to the given value.
    pub fn lowest_equivalent_value(&self, value: u64) -> u64 {
        unsafe { hdr_lowest_equivalent_value(self.histo, value as int64_t) as u64 }
    }

    /// Count for a specific value.
    pub fn count_at_value(&self, value: u64) -> u64 {
        unsafe { hdr_count_at_value(self.histo, value as int64_t) as u64 }
    }

    /// Count at a given index. (XXX safe?)
    #[allow(dead_code)]
    fn count_at_index(&self, index: u32) -> u64 {
        unsafe { hdr_count_at_index(self.histo, index as int32_t) as u64 }
    }

    /// Value of a given index. (XXX safe?)
    #[allow(dead_code)]
    fn value_at_index(&self, index: u32) -> u64 {
        unsafe { hdr_value_at_index(self.histo, index as int32_t) as u64 }
    }

    /// Linear iterator over values. Results are returned in equally weighted buckets.
    ///
    /// ```
    /// # use hdrhistogram_c::Histogram;
    /// let mut h = Histogram::init(1, 100000, 3).unwrap();
    /// for i in 1..100 { h.record_values(i, i); }
    /// for (i, c) in h.linear_iter(1).enumerate() {    // 100 buckets
    ///     # assert_eq!(i+1, c.count_added_in_this_iteration_step as usize);
    ///     println!("bucket {} = {}", i, c.count_added_in_this_iteration_step);
    /// }
    /// ```
    pub fn linear_iter<'a>(&'a self, value_units_per_bucket: u64) -> LinearIter<'a> {
        let mut ret = LinearIter { iter: Default::default(), histo: PhantomData };
        unsafe { hdr_iter_linear_init(&mut ret.iter, self.histo, value_units_per_bucket as int64_t) };
        ret
    }

    /// Logarithmic iterator over values. Results are returned in logarithmically weighted buckets.
    pub fn log_iter<'a>(&'a self, value_units_per_bucket: u64, log_base: f64) -> LogIter<'a> {
        let mut ret = LogIter { iter: Default::default(), histo: PhantomData };
        unsafe { hdr_iter_log_init(&mut ret.iter, self.histo, value_units_per_bucket as int64_t, log_base) };
        ret
    }

    /// Iterator over recorded values.
    pub fn recorded_iter<'a>(&'a self) -> RecordedIter<'a> {
        let mut ret = RecordedIter { iter: Default::default(), histo: PhantomData };
        unsafe { hdr_iter_recorded_init(&mut ret.iter, self.histo) };
        ret
    }

    /// Iterator over percentiles.
    pub fn percentile_iter<'a>(&'a self, ticks_per_half_distance: u32) -> PercentileIter<'a> {
        let mut ret = PercentileIter { iter: Default::default(), histo: PhantomData };
        unsafe { hdr_iter_percentile_init(&mut ret.iter, self.histo, ticks_per_half_distance as int32_t) };
        ret
    }

    /// Encode `Histogram` state into a Base64 encoded string.
    pub fn encode(&self) -> Result<String> {
        let mut p : *mut c_char = ptr::null_mut();
        let r = unsafe { hdr_log_encode(self.histo, &mut p) };

        if r != 0 || p.is_null() {
            Err(HistogramErr(str::from_utf8(unsafe { CStr::from_ptr(hdr_strerror(r)) }.to_bytes()).unwrap()))
        } else {
            let sz = unsafe { libc::strlen(p) as usize };
            let s = unsafe {
                let mut v = Vec::with_capacity(sz);
                v.set_len(sz);
                ptr::copy(p as *const u8, v.as_mut_ptr(), sz);
                libc::free(p as *mut c_void);
                    
                String::from_utf8_unchecked(v)
            };
            
            Ok(s)
        }
    }

    /// Decode `Histogram` state from a Base64 string generated by `encode`.
    pub fn decode(base64: &String) -> Result<Histogram> {
        let bytes = base64.as_bytes();
        let mut h : *mut hdr_histogram = ptr::null_mut();
        let r = unsafe { hdr_log_decode(&mut h, bytes.as_ptr() as *const c_char, bytes.len() as size_t) };

        if r != 0 || h.is_null() {
            Err(HistogramErr(str::from_utf8(unsafe { CStr::from_ptr(hdr_strerror(r)) }.to_bytes()).unwrap()))
        } else {
            Ok(Histogram { histo: h, owned: true })
        }
    }

    /// Return allocation size.
    fn get_memory_size(&self) -> usize { unsafe { hdr_get_memory_size(self.histo) as usize } }

    /// Return number of counters.
    #[cfg(test)]
    fn get_counts_len(&self) -> u32 { unsafe { (*self.histo).counts_len as u32 } }
}

impl Drop for Histogram {
    fn drop(&mut self) {
        if self.owned && !self.histo.is_null() {
            unsafe { libc::free(self.histo as *mut c_void) }
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        let sz = self.get_memory_size();
        let p = unsafe { libc::malloc(sz as size_t) as *mut hdr_histogram };

        if p.is_null() { panic!("allocation of hdr_histogram failed"); }

        unsafe { ptr::copy(self.histo as *const u8, p as *mut u8, sz) };

        Histogram { histo: p, owned: true }
    }
}

/// Iterator result producing counts.
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Debug)]
pub struct CountIterItem {
    /// The count of recorded values in the histogram that were added to the `total_count_to_this_value`
    /// (below) as a result on this iteration step. Since multiple iteration steps may occur with
    /// overlapping equivalent value ranges, the count may be lower than the count found at the
    /// value (e.g. multiple linear steps or percentile levels can occur within a single equivalent
    /// value range)
    pub count_added_in_this_iteration_step: u64,

    /// The sum of all recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub count: u64,

    /// The actual value level that was iterated to by the iterator
    pub value: u64,

    /// Highest value equivalent to `value`.
    pub highest_equivalent_value: u64,

    /// Median value equivalent to `value`.
    pub median_equivalent_value: u64,

    /// Lowest value equivalent to `value`.
    pub lowest_equivalent_value: u64,

    value_iterated_from: u64,
    value_iterated_to: u64,
}

/// Iterator result producing percentiles.
#[derive(PartialEq,PartialOrd,Clone,Copy,Debug)]
pub struct PercentileIterItem {
    /// The percentile of recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub percentile: f64,

    /// The sum of all recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub count: u64,

    /// The actual value level that was iterated to by the iterator
    pub value: u64,

    /// Highest value equivalent to `value`.
    pub highest_equivalent_value: u64,

    /// Median value equivalent to `value`.
    pub median_equivalent_value: u64,

    /// Lowest value equivalent to `value`.
    pub lowest_equivalent_value: u64,

    value_iterated_from: u64,
    value_iterated_to: u64,
}

/// Iterator over `Histogram` producing linear buckets.
pub struct LinearIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for LinearIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let lin : &hdr_iter_linear = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: lin.count_added_in_this_iteration_step as u64,
                                 count: self.iter.count as u64,
                                 value: self.iter.value as u64,
                                 highest_equivalent_value: self.iter.highest_equivalent_value as u64,
                                 lowest_equivalent_value: self.iter.lowest_equivalent_value as u64,
                                 median_equivalent_value: self.iter.median_equivalent_value as u64,
                                 value_iterated_from: self.iter.value_iterated_from as u64,
                                 value_iterated_to: self.iter.value_iterated_to as u64 })
        } else {
            None
        }
    }
}

/// Iterator over `Histogram` producing logarithmic buckets.
pub struct LogIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for LogIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let log : &hdr_iter_log = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: log.count_added_in_this_iteration_step as u64,
                                 count: self.iter.count as u64,
                                 value: self.iter.value as u64,
                                 highest_equivalent_value: self.iter.highest_equivalent_value as u64,
                                 lowest_equivalent_value: self.iter.lowest_equivalent_value as u64,
                                 median_equivalent_value: self.iter.median_equivalent_value as u64,
                                 value_iterated_from: self.iter.value_iterated_from as u64,
                                 value_iterated_to: self.iter.value_iterated_to as u64 })
        } else {
            None
        }
    }
}

/// Iterator over `Histogram` producing recorded values.
pub struct RecordedIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for RecordedIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let rec : &hdr_iter_recorded = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: rec.count_added_in_this_iteration_step as u64,
                                 count: self.iter.count as u64,
                                 value: self.iter.value as u64,
                                 highest_equivalent_value: self.iter.highest_equivalent_value as u64,
                                 lowest_equivalent_value: self.iter.lowest_equivalent_value as u64,
                                 median_equivalent_value: self.iter.median_equivalent_value as u64,
                                 value_iterated_from: self.iter.value_iterated_from as u64,
                                 value_iterated_to: self.iter.value_iterated_to as u64 })
        } else {
            None
        }
    }
}

/// Iterator over `Histogram` producing percentile buckets.
pub struct PercentileIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for PercentileIter<'a> {
    type Item = PercentileIterItem;
    
    fn next(&mut self) -> Option<PercentileIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let perc : &hdr_iter_percentiles = unsafe { mem::transmute(&self.iter.union) };
            
            Some(PercentileIterItem { percentile: perc.percentile,
                                      count: self.iter.count as u64,
                                      value: self.iter.value as u64,
                                      highest_equivalent_value: self.iter.highest_equivalent_value as u64,
                                      lowest_equivalent_value: self.iter.lowest_equivalent_value as u64,
                                      median_equivalent_value: self.iter.median_equivalent_value as u64,
                                      value_iterated_from: self.iter.value_iterated_from as u64,
                                      value_iterated_to: self.iter.value_iterated_to as u64 })
        } else {
            None
        }
    }
}
