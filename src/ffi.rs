extern crate libc;
use self::libc::{int64_t, int32_t, c_double, c_int, c_void, size_t, free};
use std::marker::PhantomData;
use std::mem;
use std::ptr;

#[repr(C)]
#[derive(Debug)]
pub struct hdr_histogram {
    lowest_trackable_value: int64_t,
    highest_trackable_value: int64_t,
    unit_magnitude: int32_t,
    pub significant_figures: int64_t,
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
    pub total_count: int64_t,
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
    value_units_first_bucket: int64_t,
    log_base: c_double,
    count_added_in_this_iteration_step: int64_t,
    next_value_reporting_level: int64_t,
    next_value_reporting_level_lowest_equivalent: int64_t,
}

#[allow(raw_pointer_derive)]
#[repr(C)]
#[derive(Debug)]
struct hdr_iter {
    h: *const hdr_histogram,
    
    bucket_index: int32_t,
    sub_bucket_index: int32_t,
    count_at_index: int64_t,
    count_to_index: int64_t,
    value_from_index: int64_t,
    highest_equivalent_value: int64_t,

    union: [int64_t; 5],        // hdr_iter_log

    _next_fp: *const c_void,    // function pointer
}

impl Default for hdr_iter {
    fn default() -> Self { unsafe { mem::zeroed() } }
}

#[allow(dead_code)]
#[link(name = "hdr_histogram")]
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
}

#[derive(Copy,Clone,Debug)]
pub struct HistogramBucketCfg {
    cfg: hdr_histogram_bucket_config
}

impl HistogramBucketCfg {
    pub fn new(lowest_trackable_value: u64, highest_trackable_value: u64, significant_figures: u32) -> Result<HistogramBucketCfg, HistoErr> {
        let mut ret = HistogramBucketCfg { cfg: Default::default() };
        let r = unsafe {
            hdr_calculate_bucket_config(lowest_trackable_value as int64_t,
                                        highest_trackable_value as int64_t,
                                        significant_figures as c_int,
                                        &mut ret.cfg)
        };

        if r == 0 {
            Ok(ret)
        } else {
            Err(HistoErr)
        }
    }
}

#[derive(Debug)]
pub struct HistoErr;

pub struct Histogram {
    histo: *mut hdr_histogram,
}

/// Instance of a Histogram.
impl Histogram {
    /// Create a new Histogram instance. `lowest_trackable_value`..`highest_trackable_value` defines
    /// the range of values in the `Histogram`. `lowest_trackable_value` must be >= 1.
    /// `significant_figures` defines the precision, and must be in the range 1..5.
    ///
    /// `HistoErr` is the catch-all failure case. It doesn't report much detail because the
    /// underlying library doesn't.
    pub fn init(lowest_trackable_value: u64, highest_trackable_value: u64, significant_figures: u32) -> Result<Histogram, HistoErr> {
        let mut histo : *mut hdr_histogram = ptr::null_mut();
        let r = unsafe {
            hdr_init(lowest_trackable_value as int64_t, highest_trackable_value as int64_t,
                     significant_figures as int32_t, &mut histo)
        };

        if r != 0 || histo.is_null() {
            Err(HistoErr)
        } else {
            Ok(Histogram { histo: histo })
        }
    }

    /// Zero all histogram state in place.
    pub fn reset(&mut self) { unsafe { hdr_reset(self.histo) } }

    /// Return allocation size.
    pub fn get_memory_size(&self) -> usize { unsafe { hdr_get_memory_size(self.histo) as usize } }

    /// Return number of counters.
    pub fn get_counts_len(&self) -> u32 { unsafe { (*self.histo).counts_len as u32 } }

    /// Total of all counters
    pub fn total_count(&self) -> u64 { unsafe { (*self.histo).total_count as u64 } }

    /// Record a specific value. Returns true if successful.
    pub fn record_value(&mut self, value: u64) -> bool {
        unsafe { hdr_record_value(self.histo, value as int64_t) }
    }

    /// Record multiple counts of a specific value. Returns true if successful.
    pub fn record_values(&mut self, value: u64, count: u64) -> bool {
        unsafe { hdr_record_values(self.histo, value as int64_t, count as int64_t) }
    }

    /// Record a value, correcting for coordinated omission. This should be used when accumulating
    /// latency measurements taked on a regular timebase (expected_interval). Any latency that's
    /// well above that interval implies some kind of outage in which sampled were lost. This
    /// corrects for those lost samples to preserve the integrity of the overall statistics.
    pub fn record_corrected_value(&mut self, value: u64, expected_interval: u64) -> bool {
        unsafe { hdr_record_corrected_value(self.histo, value as int64_t, expected_interval as int64_t) }
    }

    /// As with `record_corrected_value()` but multiple counts of the value.
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

    /// Smallest recorded value.
    pub fn min(&self) -> u64 {
        unsafe { hdr_min(self.histo) as u64 }
    }

    /// Largest recorded value.
    pub fn max(&self) -> u64 {
        unsafe { hdr_max(self.histo) as u64 }
    }

    /// Value at a particular percentile (0-100).
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
}

impl Drop for Histogram {
    fn drop(&mut self) {
        if !self.histo.is_null() {
            unsafe { libc::free(self.histo as *mut c_void) }
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        let sz = self.get_memory_size();
        let p = unsafe { libc::malloc(sz as size_t) as *mut hdr_histogram };

        if p.is_null() { panic!("allocation of hdr_histogram failed"); }

        unsafe { ptr::copy(self.histo, p, sz) };

        Histogram { histo: p }
    }
}
            

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Debug)]
pub struct CountIterItem {
    pub count_added_in_this_iteration_step: u64,
}

#[derive(PartialEq,PartialOrd,Clone,Copy,Debug)]
pub struct PercentileIterItem {
    pub percentile: f64,
}

pub struct LinearIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for LinearIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let lin : &hdr_iter_linear = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: lin.count_added_in_this_iteration_step as u64 })
        } else {
            None
        }
    }
}

pub struct LogIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for LogIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let log : &hdr_iter_log = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: log.count_added_in_this_iteration_step as u64 })
        } else {
            None
        }
    }
}

pub struct RecordedIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for RecordedIter<'a> {
    type Item = CountIterItem;
    
    fn next(&mut self) -> Option<CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let rec : &hdr_iter_recorded = unsafe { mem::transmute(&self.iter.union) };
            
            Some(CountIterItem { count_added_in_this_iteration_step: rec.count_added_in_this_iteration_step as u64 })
        } else {
            None
        }
    }
}

pub struct PercentileIter<'a> {
    iter: hdr_iter,
    histo: PhantomData<&'a Histogram>,
}

impl<'a> Iterator for PercentileIter<'a> {
    type Item = PercentileIterItem;
    
    fn next(&mut self) -> Option<PercentileIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let perc : &hdr_iter_percentiles = unsafe { mem::transmute(&self.iter.union) };
            
            Some(PercentileIterItem { percentile: perc.percentile })
        } else {
            None
        }
    }
}
