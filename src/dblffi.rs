extern crate libc;
use self::libc::{int64_t, int32_t, c_double, c_int, c_void, free};
use std::ptr;
use ffi::{hdr_histogram, HistoErr};

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
    histo: *mut hdr_dbl_histogram
}

impl F64Histogram {
    pub fn init(highest_to_lowest_ratio: i64, significant_figures: u32) -> Result<F64Histogram, HistoErr> {
        let mut histo : *mut hdr_dbl_histogram = ptr::null_mut();
        let r = unsafe {
            hdr_dbl_init(highest_to_lowest_ratio as int64_t, significant_figures as int32_t, &mut histo)
        };

        if r != 0 || histo.is_null() {
            Err(HistoErr)
        } else {
            Ok(F64Histogram { histo: histo })
        }
    }

    pub fn significant_figures(&self) -> u32 { unsafe { (*self.histo).values.significant_figures as u32 } }
    pub fn highest_to_lowest_value_ratio(&self) -> i64 { unsafe { (*self.histo).highest_to_lowest_value_ratio as i64 } }
    pub fn current_lowest_value(&self) -> f64 { unsafe { (*self.histo).current_lowest_value as f64 } }
    pub fn total_count(&self) -> u64 { unsafe { (*self.histo).values.total_count as u64 } }
    
    pub fn reset(&mut self) { unsafe { hdr_dbl_reset(self.histo) } }
    
    pub fn record_value(&mut self, value: f64) -> bool {
        unsafe { hdr_dbl_record_value(self.histo, value) }
    }
    pub fn record_values(&mut self, value: f64, count: u64) -> bool {
        unsafe { hdr_dbl_record_values(self.histo, value, count as int64_t) }
    }
    pub fn record_corrected_value(&mut self, value: f64, expected_interval: f64) -> bool {
        unsafe { hdr_dbl_record_corrected_value(self.histo, value, expected_interval) }
    }
    pub fn record_corrected_values(&mut self, value: f64, count: u64, expected_interval: f64) -> bool {
        unsafe { hdr_dbl_record_corrected_values(self.histo, value, count as int64_t, expected_interval) }
    }

    pub fn size_of_equivalent_value_range(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_size_of_equivalent_value_range(self.histo, value) }
    }

    pub fn lowest_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_lowest_equivalent_value(self.histo, value) }
    }
    pub fn highest_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_highest_equivalent_value(self.histo, value) }
    }    
    pub fn median_equivalent_value(&self, value: f64) -> f64 {
        unsafe { hdr_dbl_median_equivalent_value(self.histo, value) }
    }
    
    pub fn values_are_equivalent(&self, a: f64, b: f64) -> bool {
        unsafe { hdr_dbl_values_are_equivalent(self.histo, a, b) }
    }

    pub fn mean(&self) -> f64 { unsafe { hdr_dbl_mean(self.histo) } }
    pub fn min(&self) -> f64 { unsafe { hdr_dbl_min(self.histo) } }
    pub fn max(&self) -> f64 { unsafe { hdr_dbl_max(self.histo) } }
    pub fn stddev(&self) -> f64 { unsafe { hdr_dbl_stddev(self.histo) } }
    pub fn value_at_percentile(&self, percentile: f64) -> f64 {
        unsafe { hdr_dbl_value_at_percentile(self.histo, percentile) }
    }
    pub fn count_at_value(&self, value: f64) -> u64 {
        unsafe { hdr_dbl_count_at_value(self.histo, value) as u64 }
    }

    pub fn add(&mut self, other: &F64Histogram) -> u64 {
        unsafe { hdr_dbl_add(self.histo, other.histo) as u64 }
    }

    //pub fn add_while_correcting_for_coordinated_omission(...)
}

impl Drop for F64Histogram {
    fn drop(&mut self) {
        if !self.histo.is_null() {
            unsafe { libc::free(self.histo as *mut c_void) }
        }
    }
}
