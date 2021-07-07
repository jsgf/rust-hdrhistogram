//! # Rust binding for HdrHistogram_c
//!
//! This crate implements bindings for
//! [HdrHistogram_c](https://github.com/hdrhistogram/HdrHistogram_c), a flexible library for
//! recording histograms without having to know very much about the data being histogrammed.
//!
//! The top-level type is [`Histogram`](struct.Histogram.html).
//!
//! # Example
//!
//! This sets up a histogram to record values in the range 1..1000_000 with 2 significant figures of
//! precision. It then records one count each of 1 and 10, and 40 counts of 100.
//!
//! ```
//! # use hdrhistogram_c::Histogram;
//! let mut h = Histogram::new(1, 1000000, 2).unwrap();
//!
//! h.record_value(1);
//! h.record_value(10);
//! h.record_values(100, 40);
//!
//! assert_eq!(h.total_count(), 42);
//! assert_eq!(h.min(), 1);
//! assert_eq!(h.max(), 100);
//! ```

#![feature(maybe_uninit_slice, vec_split_at_spare)]

use libc::{c_char, c_void};
use paste::paste;
use std::{ffi::CStr, mem::MaybeUninit, ptr, str};
use thiserror::Error;

// mod ffi;

#[allow(dead_code)]
#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("HdrHistogram_c/src/hdr_histogram.h");
        include!("HdrHistogram_c/src/hdr_histogram_log.h");
        include!("src/glue.h");

        type hdr_histogram;

        unsafe fn hdr_init(
            lowest_discernible_value: i64,
            highest_trackable_value: i64,
            significant_figures: i32,
            result: *mut *mut hdr_histogram,
        ) -> i32;

        unsafe fn hdr_close(hdr: *mut hdr_histogram);
        unsafe fn hdr_reset(hdr: *mut hdr_histogram);
        unsafe fn hdr_get_memory_size(hdr: *mut hdr_histogram) -> usize;

        unsafe fn hdr_record_value(hdr: *mut hdr_histogram, value: i64) -> bool;
        unsafe fn hdr_record_value_atomic(hdr: *mut hdr_histogram, value: i64) -> bool;
        unsafe fn hdr_record_values(hdr: *mut hdr_histogram, value: i64, count: i64) -> bool;
        unsafe fn hdr_record_values_atomic(hdr: *mut hdr_histogram, value: i64, count: i64)
            -> bool;
        unsafe fn hdr_record_corrected_value(
            hdr: *mut hdr_histogram,
            value: i64,
            expected_interval: i64,
        ) -> bool;
        unsafe fn hdr_record_corrected_value_atomic(
            hdr: *mut hdr_histogram,
            value: i64,
            expected_interval: i64,
        ) -> bool;
        unsafe fn hdr_record_corrected_values(
            hdr: *mut hdr_histogram,
            value: i64,
            count: i64,
            expected_interval: i64,
        ) -> bool;
        unsafe fn hdr_record_corrected_values_atomic(
            hdr: *mut hdr_histogram,
            value: i64,
            count: i64,
            expected_interval: i64,
        ) -> bool;

        unsafe fn hdr_add(hdr: *mut hdr_histogram, other: *const hdr_histogram) -> i64;
        unsafe fn hdr_add_while_correcting_for_coordinated_omission(
            hdr: *mut hdr_histogram,
            other: *mut hdr_histogram,
            expected_interval: i64,
        ) -> i64;
        unsafe fn hdr_min(hdr: *const hdr_histogram) -> i64;
        unsafe fn hdr_max(hdr: *const hdr_histogram) -> i64;
        unsafe fn hdr_value_at_percentile(hdr: *const hdr_histogram, percentile: f64) -> i64;
        unsafe fn hdr_value_at_percentiles(
            hdr: *const hdr_histogram,
            percentiles: *const f64,
            values: *mut i64,
            length: usize,
        ) -> i32;
        unsafe fn hdr_stddev(hdr: *const hdr_histogram) -> f64;
        unsafe fn hdr_mean(hdr: *const hdr_histogram) -> f64;
        unsafe fn hdr_count_at_value(hdr: *const hdr_histogram, value: i64) -> i64;
        unsafe fn hdr_count_at_index(hdr: *const hdr_histogram, index: i32) -> i64;
        unsafe fn hdr_value_at_index(hdr: *const hdr_histogram, index: i32) -> i64;
        unsafe fn hdr_values_are_equivalent(hdr: *const  hdr_histogram, a: i64, b: i64) -> bool;
        unsafe fn hdr_lowest_equivalent_value(hdr: *const  hdr_histogram, value: i64) -> i64;

        unsafe fn hdr_log_encode(hdr: *mut hdr_histogram, encoded: *mut *mut c_char) -> i32;
        unsafe fn hdr_log_decode(
            hdr: *mut *mut hdr_histogram,
            base64_histogram: *mut c_char,
            base64_len: usize,
        ) -> i32;

        unsafe fn hdr_strerror(err: i32) -> *const c_char;

        // Rust accessor glue
        unsafe fn hdr_rust_total_count(hdr: *const hdr_histogram) -> i64;
        unsafe fn hdr_rust_counts_len(hdr: *const hdr_histogram) -> i64;
        unsafe fn hdr_rust_clone(hdr: *const hdr_histogram) -> *mut hdr_histogram;
    }
}

pub struct Histogram(*mut ffi::hdr_histogram);

#[derive(Error, Debug)]
pub enum HistogramErr {
    #[error("Allocation failed")]
    AllocFail,
    #[error("Initialization failed")]
    InitFailed,
    #[error("Encoding/Decoding failed: {}", _0)]
    CodecFailed(&'static str),
}

unsafe impl Send for Histogram {}

macro_rules! ffi {
    (mut $name: ident $( ( $( $param:ident : $pty:ty ),* ) )? $(-> $return: ty)?) => {
        paste! {
            #[inline]
            pub fn $name ( &mut self $(, $($param: $pty,)* )? ) $(-> $return)? {
                unsafe {
                    ffi::[<hdr_ $name>](self.0 $(, $($param,)* )? )
                }
            }
        }
    };
    ($name: ident $( ( $( $param:ident : $pty:ty ),* ) )? $(-> $return: ty)?) => {
        paste! {
            #[inline]
            pub fn $name ( &self $(, $($param: $pty,)* )? ) $(-> $return)? {
                unsafe {
                    ffi::[<hdr_ $name>](self.0 $(, $($param,)* )? )
                }
            }
        }
    };
}

impl Histogram {
    pub fn new(
        lowest_discernible_value: i64,
        highest_trackable_value: i64,
        significant_figures: i32,
    ) -> Result<Self, HistogramErr> {
        let mut ret: *mut ffi::hdr_histogram = ptr::null_mut();
        unsafe {
            let res = ffi::hdr_init(
                lowest_discernible_value,
                highest_trackable_value,
                significant_figures,
                &mut ret,
            );

            if res == libc::ENOMEM || ret.is_null() {
                return Err(HistogramErr::AllocFail);
            }

            if res != 0 {
                return Err(HistogramErr::InitFailed);
            };
        }

        Ok(Histogram(ret))
    }

    ffi!(mut reset);
    ffi!(get_memory_size -> usize);

    ffi!(mut record_value(value: i64) -> bool);
    ffi!(mut record_values(value: i64, count: i64) -> bool);
    ffi!(mut record_corrected_value(value: i64, expected_interval: i64) -> bool);
    ffi!(mut record_corrected_values(value: i64, count: i64, expected_interval: i64) -> bool);

    ffi!(min -> i64);
    ffi!(max -> i64);
    ffi!(stddev -> f64);
    ffi!(mean -> f64);

    ffi!(value_at_percentile(percentile: f64) -> i64);
    ffi!(count_at_value(value: i64) -> i64);
    //ffi!(count_at_index(index: i32) -> i64);
    //ffi!(value_at_index(index: i32) -> i64);
    ffi!(values_are_equivalent(a: i64, b: i64) -> bool);
    ffi!(lowest_equivalent_value(value: i64) -> i64);

    pub fn value_at_percentiles(&self, percentiles: &[f64]) -> Box<[i64]> {
        let mut values: Vec<i64> = Vec::with_capacity(percentiles.len());
        let (_, uninit) = values.split_at_spare_mut();

        let res = unsafe {
            ffi::hdr_value_at_percentiles(
                self.0,
                percentiles.as_ptr(),
                MaybeUninit::slice_as_mut_ptr(uninit),
                percentiles.len(),
            )
        };

        assert_eq!(res, 0, "hdr_value_at_percentiles invalid pointer?");

        // SAFETY: values have now been initialized
        unsafe { values.set_len(percentiles.len()) };

        values.into_boxed_slice()
    }

    pub fn add(&mut self, other: &Histogram) -> i64 {
        unsafe { ffi::hdr_add(self.0, other.0) }
    }

    pub fn add_while_correcting_for_coordinated_omission(
        &mut self,
        other: &Histogram,
        expected_interval: i64,
    ) -> i64 {
        unsafe {
            ffi::hdr_add_while_correcting_for_coordinated_omission(
                self.0,
                other.0,
                expected_interval,
            )
        }
    }

    pub fn total_count(&self) -> i64 {
        unsafe { ffi::hdr_rust_total_count(self.0) }
    }

    pub fn get_counts_len(&self) -> i64 {
        unsafe { ffi::hdr_rust_counts_len(self.0) }
    }

    /// Encode `Histogram` state into a Base64 encoded string.
    pub fn encode(&self) -> Result<String, HistogramErr> {
        let mut p: *mut c_char = ptr::null_mut();
        let r = unsafe { ffi::hdr_log_encode(self.0, &mut p) };

        if r != 0 || p.is_null() {
            Err(HistogramErr::CodecFailed(
                str::from_utf8(unsafe { CStr::from_ptr(ffi::hdr_strerror(r)) }.to_bytes()).unwrap(),
            ))
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
    pub fn decode(base64: &String) -> Result<Histogram, HistogramErr> {
        let bytes = base64.as_bytes();
        let mut h: *mut ffi::hdr_histogram = ptr::null_mut();
        let r =
            unsafe { ffi::hdr_log_decode(&mut h, bytes.as_ptr() as *mut c_char, bytes.len()) };

        if r != 0 || h.is_null() {
            Err(HistogramErr::CodecFailed(
                str::from_utf8(unsafe { CStr::from_ptr(ffi::hdr_strerror(r)) }.to_bytes()).unwrap(),
            ))
        } else {
            Ok(Histogram(h))
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        let new = unsafe { ffi::hdr_rust_clone(self.0) };
        assert!(!new.is_null(), "Clone allocation failed");

        Histogram(new)
    }
}

impl Drop for Histogram {
    fn drop(&mut self) {
        unsafe { ffi::hdr_close(self.0) }
    }
}

#[cfg(test)]
mod test;

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_destroy() {
        let hdr = Histogram::new(1, 1000, 5).unwrap();

        println!("mem use {}", hdr.get_memory_size());
    }
} */

//pub use ffi::{Histogram, HistogramErr, LinearIter, LogIter, PercentileIter, RecordedIter,
//              CountIterItem, PercentileIterItem };

// /// Result from operations which may fail.
//pub type Result<T> = std::result::Result<T, HistogramErr>;
