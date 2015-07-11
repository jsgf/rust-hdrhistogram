extern crate libc;
use self::libc::{int64_t, int32_t, c_double, c_int, c_void, size_t, free};
use std::ptr;
use std::mem;
use ffi::{hdr_histogram, HistoErr,
          hdr_iter_log_init, hdr_iter_linear_init, hdr_iter_percentile_init, hdr_iter_recorded_init, hdr_iter_next,
          hdr_iter, hdr_iter_linear, hdr_iter_percentiles, hdr_iter_log, hdr_iter_recorded};

use super::Histogram;

#[cfg(test)]
mod test;

#[repr(C)]
struct hdr_dbl_histogram {
    current_lowest_value: c_double,
    current_highest_value: c_double,
    highest_to_lowest_value_ratio: int64_t,
    int_to_dbl_conversion_ratio: c_double,
    dbl_to_int_conversion_ratio: c_double,

    values: hdr_histogram,
}

#[allow(dead_code)]
#[link(name="hdr_histogram")]
extern {
    fn hdr_dbl_init(highest_to_lowest_value_ratio: int64_t, significant_figures: int32_t, result: *mut *mut hdr_dbl_histogram) -> c_int;

    fn hdr_dbl_record_value(h: *mut hdr_dbl_histogram, value: c_double) -> bool;
    fn hdr_dbl_record_values(h: *mut hdr_dbl_histogram, value: c_double, count: int64_t) -> bool;
    fn hdr_dbl_record_corrected_value(h: *mut hdr_dbl_histogram, value: c_double, expected_interval: c_double) -> bool;
    fn hdr_dbl_record_corrected_values(h: *mut hdr_dbl_histogram, value: c_double, count: int64_t, expected_interval: c_double) -> bool;

    fn hdr_dbl_size_of_equivalent_value_range(h: *const hdr_dbl_histogram, value: c_double) -> c_double;
    fn hdr_dbl_lowest_equivalent_value(h: *const hdr_dbl_histogram, value: c_double) -> c_double;
    fn hdr_dbl_highest_equivalent_value(h: *const hdr_dbl_histogram, value: c_double) -> c_double;
    fn hdr_dbl_median_equivalent_value(h: *const hdr_dbl_histogram, value: c_double) -> c_double;
    fn hdr_dbl_values_are_equivalent(h: *const hdr_dbl_histogram, a: c_double, b: c_double) -> bool;
    
    fn hdr_dbl_add_while_correcting_for_coordinated_omission(dest: *mut *mut hdr_dbl_histogram, src: *const hdr_dbl_histogram,
                                                             expected_interval: c_double) -> int64_t;
    fn hdr_dbl_mean(h: *const hdr_dbl_histogram) -> c_double;
    fn hdr_dbl_value_at_percentile(h: *const hdr_dbl_histogram, percentile: c_double) -> c_double;
    fn hdr_dbl_min(h: *const hdr_dbl_histogram) -> c_double;
    fn hdr_dbl_max(h: *const hdr_dbl_histogram) -> c_double;
    fn hdr_dbl_stddev(h: *const hdr_dbl_histogram) -> c_double;

    fn hdr_dbl_add(sum: *mut hdr_dbl_histogram, addend: *const hdr_dbl_histogram) -> int64_t;
    fn hdr_dbl_reset(h: *mut hdr_dbl_histogram);
    fn hdr_dbl_count_at_value(h: *const hdr_dbl_histogram, value: c_double) -> int64_t;
}

pub struct F64Histogram {
    dblhisto: *mut hdr_dbl_histogram,
    histo: Histogram,
}

impl F64Histogram {
    pub fn init(highest_to_lowest_ratio: i64, significant_figures: u32) -> Result<F64Histogram, HistoErr> {
        let mut dblhisto : *mut hdr_dbl_histogram = ptr::null_mut();
        let r = unsafe {
            hdr_dbl_init(highest_to_lowest_ratio as int64_t, significant_figures as int32_t, &mut dblhisto)
        };

        if r != 0 || dblhisto.is_null() {
            Err(HistoErr)
        } else {
            Ok(F64Histogram { dblhisto: dblhisto, histo: Histogram::prealloc(unsafe { &mut (*dblhisto).values }) })
        }
    }

    fn int_to_dbl(&self) -> f64 {
        unsafe { (*self.dblhisto).int_to_dbl_conversion_ratio }
    }
    
    pub fn significant_figures(&self) -> u32 { unsafe { (*self.dblhisto).values.significant_figures as u32 } }
    pub fn highest_to_lowest_value_ratio(&self) -> i64 { unsafe { (*self.dblhisto).highest_to_lowest_value_ratio as i64 } }
    pub fn current_lowest_value(&self) -> f64 { unsafe { (*self.dblhisto).current_lowest_value as f64 } }
    pub fn total_count(&self) -> u64 { unsafe { (*self.dblhisto).values.total_count as u64 } }
    
    pub fn reset(&mut self) { unsafe { hdr_dbl_reset(self.dblhisto) } }
    
    pub fn record_value(&mut self, value: f64) -> bool {
        unsafe { hdr_dbl_record_value(self.dblhisto, value) }
    }
    pub fn record_values(&mut self, value: f64, count: u64) -> bool {
        unsafe { hdr_dbl_record_values(self.dblhisto, value, count as int64_t) }
    }
    pub fn record_corrected_value(&mut self, value: f64, expected_interval: f64) -> bool {
        unsafe { hdr_dbl_record_corrected_value(self.dblhisto, value, expected_interval) }
    }
    pub fn record_corrected_values(&mut self, value: f64, count: u64, expected_interval: f64) -> bool {
        unsafe { hdr_dbl_record_corrected_values(self.dblhisto, value, count as int64_t, expected_interval) }
    }

    pub fn size_of_equivalent_value_range(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_size_of_equivalent_value_range(self.dblhisto, value) }
    }

    pub fn lowest_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_lowest_equivalent_value(self.dblhisto, value) }
    }
    pub fn highest_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_highest_equivalent_value(self.dblhisto, value) }
    }    
    pub fn median_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_median_equivalent_value(self.dblhisto, value) }
    }
    
    pub fn values_are_equivalent(&self, a: f64, b: f64) -> bool {
        unsafe { hdr_dbl_values_are_equivalent(self.dblhisto, a, b) }
    }

    pub fn mean(&self) -> f64 { unsafe { hdr_dbl_mean(self.dblhisto) } }
    pub fn min(&self) -> f64 { unsafe { hdr_dbl_min(self.dblhisto) } }
    pub fn max(&self) -> f64 { unsafe { hdr_dbl_max(self.dblhisto) } }
    pub fn stddev(&self) -> f64 { unsafe { hdr_dbl_stddev(self.dblhisto) } }
    pub fn value_at_percentile(&self, percentile: f64) -> f64 {
        unsafe { hdr_dbl_value_at_percentile(self.dblhisto, percentile) }
    }
    pub fn count_at_value(&self, value: f64) -> u64 {
        unsafe { hdr_dbl_count_at_value(self.dblhisto, value) as u64 }
    }

    pub fn add(&mut self, other: &F64Histogram) -> u64 {
        unsafe { hdr_dbl_add(self.dblhisto, other.dblhisto) as u64 }
    }

    pub fn linear_iter<'a>(&'a self, value_units_per_bucket: u64) -> F64LinearIter<'a> {
        let mut ret = F64LinearIter { iter: Default::default(), histo: self };
        unsafe { hdr_iter_linear_init(&mut ret.iter, self.histo.histo, value_units_per_bucket as int64_t) };
        ret
    }

    pub fn log_iter<'a>(&'a self, value_units_per_bucket: u64, log_base: f64) -> F64LogIter<'a> {
        let mut ret = F64LogIter { iter: Default::default(), histo: self };
        unsafe { hdr_iter_log_init(&mut ret.iter, self.histo.histo, value_units_per_bucket as int64_t, log_base) };
        ret
    }

    pub fn recorded_iter<'a>(&'a self) -> F64RecordedIter<'a> {
        let mut ret = F64RecordedIter { iter: Default::default(), histo: self };
        unsafe { hdr_iter_recorded_init(&mut ret.iter, self.histo.histo) };
        ret
    }

    pub fn percentile_iter<'a>(&'a self, ticks_per_half_distance: u32) -> F64PercentileIter<'a> {
        let mut ret = F64PercentileIter { iter: Default::default(), histo: self };
        unsafe { hdr_iter_percentile_init(&mut ret.iter, self.histo.histo, ticks_per_half_distance as int32_t) };
        ret
    }


    //pub fn add_while_correcting_for_coordinated_omission(...)
}


impl Drop for F64Histogram {
    fn drop(&mut self) {
        if !self.dblhisto.is_null() {
            unsafe { libc::free(self.dblhisto as *mut c_void) }
        }
    }
}

impl Clone for F64Histogram {
    fn clone(&self) -> F64Histogram {
        let sz = mem::size_of::<hdr_dbl_histogram>() + mem::size_of::<int64_t>() * unsafe { (*self.dblhisto).values.counts_len as usize };
        let p = unsafe { libc::malloc(sz as size_t) as *mut hdr_dbl_histogram };

        if p.is_null() {
            panic!("Out of memory in F64Histogram::Clone");
        }

        unsafe { ptr::copy(self.dblhisto as *const u8, p as *mut u8, sz) };

        F64Histogram { dblhisto: p, histo: Histogram::prealloc(unsafe { &mut (*p).values }) }
    }
}

#[derive(PartialEq,PartialOrd,Clone,Copy,Debug)]
pub struct F64CountIterItem {
    /// The count of recorded values in the histogram that were added to the `total_count_to_this_value`
    /// (below) as a result on this iteration step. Since multiple iteration steps may occur with
    /// overlapping equivalent value ranges, the count may be lower than the count found at the
    /// value (e.g. multiple linear steps or percentile levels can occur within a single equivalent
    /// value range)
    pub count_added_in_this_iteration_step: u64,

    /// The sum of all recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub count_to_index: u64,

    /// The actual value level that was iterated to by the iterator
    pub value_from_index: f64,

    /// Highest value equivalent to `value_from_index`.
    pub highest_equivalent_value: f64,
    
    /// The count of recorded values in the histogram that exactly match this
    /// `lowest_equivalent_value(value_from_index)`..`highest_equivalent_value(value_from_index)`
    /// value range.
    pub count_at_index: u64,
}

#[derive(PartialEq,PartialOrd,Clone,Copy,Debug)]
pub struct F64PercentileIterItem {
    /// The percentile of recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub percentile: f64,

    /// The sum of all recorded values in the histogram at values equal or smaller than `value_from_index`.
    pub count_to_index: u64,

    /// The actual value level that was iterated to by the iterator
    pub value_from_index: f64,

    /// Highest value equivalent to `value_from_index`.
    pub highest_equivalent_value: f64,
    
    /// The count of recorded values in the histogram that exactly match this
    /// `lowest_equivalent_value(value_from_index)`..`highest_equivalent_value(value_from_index)`
    /// value range.
    pub count_at_index: u64,
}

pub struct F64LinearIter<'a> {
    iter: hdr_iter,
    histo: &'a F64Histogram,
}

impl<'a> Iterator for F64LinearIter<'a> {
    type Item = F64CountIterItem;
    
    fn next(&mut self) -> Option<F64CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let lin : &hdr_iter_linear = unsafe { mem::transmute(&self.iter.union) };
            let scale = self.histo.int_to_dbl();
            
            Some(F64CountIterItem { count_added_in_this_iteration_step: lin.count_added_in_this_iteration_step as u64,
                                    value_from_index: self.iter.value_from_index as f64 * scale,
                                    highest_equivalent_value: self.iter.highest_equivalent_value as f64 * scale,
                                    count_to_index: self.iter.count_to_index as u64,
                                    count_at_index: self.iter.count_at_index as u64 })
        } else {
            None
        }
    }
}

pub struct F64LogIter<'a> {
    iter: hdr_iter,
    histo: &'a F64Histogram,
}

impl<'a> Iterator for F64LogIter<'a> {
    type Item = F64CountIterItem;
    
    fn next(&mut self) -> Option<F64CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let log : &hdr_iter_log = unsafe { mem::transmute(&self.iter.union) };
            let scale = self.histo.int_to_dbl();
            
            Some(F64CountIterItem { count_added_in_this_iteration_step: log.count_added_in_this_iteration_step as u64,
                                    value_from_index: self.iter.value_from_index as f64 * scale,
                                    highest_equivalent_value: self.iter.highest_equivalent_value as f64 * scale,
                                    count_to_index: self.iter.count_to_index as u64,
                                    count_at_index: self.iter.count_at_index as u64 })
        } else {
            None
        }
    }
}

pub struct F64RecordedIter<'a> {
    iter: hdr_iter,
    histo: &'a F64Histogram,
}

impl<'a> Iterator for F64RecordedIter<'a> {
    type Item = F64CountIterItem;
    
    fn next(&mut self) -> Option<F64CountIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let rec : &hdr_iter_recorded = unsafe { mem::transmute(&self.iter.union) };
            let scale = self.histo.int_to_dbl();
            
            Some(F64CountIterItem { count_added_in_this_iteration_step: rec.count_added_in_this_iteration_step as u64,
                                    value_from_index: self.iter.value_from_index as f64 * scale,
                                    highest_equivalent_value: self.iter.highest_equivalent_value as f64 * scale,
                                    count_to_index: self.iter.count_to_index as u64,
                                    count_at_index: self.iter.count_at_index as u64 })
        } else {
            None
        }
    }
}

pub struct F64PercentileIter<'a> {
    iter: hdr_iter,
    histo: &'a F64Histogram,
}

impl<'a> Iterator for F64PercentileIter<'a> {
    type Item = F64PercentileIterItem;
    
    fn next(&mut self) -> Option<F64PercentileIterItem> {
        if unsafe { hdr_iter_next(&mut self.iter) } {
            let perc : &hdr_iter_percentiles = unsafe { mem::transmute(&self.iter.union) };
            let scale = self.histo.int_to_dbl();
            
            Some(F64PercentileIterItem { percentile: perc.percentile,
                                         value_from_index: self.iter.value_from_index as f64 * scale,
                                         highest_equivalent_value: self.iter.highest_equivalent_value as f64 * scale,
                                         count_to_index: self.iter.count_to_index as u64,
                                         count_at_index: self.iter.count_at_index as u64 })
        } else {
            None
        }
    }
}
