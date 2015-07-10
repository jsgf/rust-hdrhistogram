use super::F64Histogram;
use ffi::test::compare_double;

const TRACKABLE_VALUE_RANGE_SIZE : i64 = 3600 * 1000 * 1000; // e.g. for 1 hr in usec units
const SIGNIFICANT_FIGURES : u32 = 3;
const TEST_VALUE_LEVEL : f64 = 4.0;

#[test]
fn test_construct_argument_ranges() {
    assert!(F64Histogram::init(1, SIGNIFICANT_FIGURES).is_err());
    assert!(F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, 0).is_err());
    assert!(F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, 6).is_err());    
}

#[test]
fn test_construction_argument_gets() {
    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(2_f64.powf(20.0)));
    assert!(h.record_value(1.0));

    assert_eq!(SIGNIFICANT_FIGURES, h.significant_figures());
    assert_eq!(TRACKABLE_VALUE_RANGE_SIZE, h.highest_to_lowest_value_ratio());
    assert!(compare_double(1.0, h.current_lowest_value(), 0.001));
}

fn ctlz64(x: u64) -> u64 {
    //    unsafe { intrinsics::ctlz64(x) }
    let mut x = x;
    let mut r = 0;

    while (x & 0x8000000000000000) == 0 && r < 64 {
        r += 1;
        x <<= 1;
    };
    r
}

#[test]
fn test_data_range() {
    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(0.0));
    assert_eq!(h.count_at_value(0.0), 1);

    let mut top_value = 1.0;
    while h.record_value(top_value) {
        top_value *= 2.0;
    }

    assert!(compare_double((1_u64 << 33) as f64, top_value, 0.00001));
    assert_eq!(h.count_at_value(0.0), 1);

    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(0.0));

    let mut bottom_value = (1_u64 << 33) as f64;
    while h.record_value(bottom_value) {
        bottom_value /= 2.0;
    }

    let expected_range = 1_u64 << ((64 - ctlz64(TRACKABLE_VALUE_RANGE_SIZE as u64)) + 1);

    assert!(compare_double(expected_range as f64, top_value / bottom_value, 0.0001));
    assert!(compare_double(bottom_value, 1.0, 0.00001));
    assert_eq!(h.count_at_value(0.0), 1);
}

#[test]
fn test_record_value() {
    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(TEST_VALUE_LEVEL));

    assert_eq!(h.count_at_value(TEST_VALUE_LEVEL), 1);
    assert_eq!(h.total_count(), 1);
}

#[test]
fn test_record_value_overflow() {
    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(TRACKABLE_VALUE_RANGE_SIZE as f64 * 3.0));
    assert!(!h.record_value(1.0));
}

#[test]
fn test_record_value_with_expected_interval() {
    let mut raw = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(raw.record_value(0.0));
    assert!(raw.record_value(TEST_VALUE_LEVEL));

    assert_eq!(raw.count_at_value(0.0), 1);
    assert_eq!(raw.count_at_value(TEST_VALUE_LEVEL * 1.0 / 4.0), 0);
    assert_eq!(raw.count_at_value(TEST_VALUE_LEVEL * 2.0 / 4.0), 0);
    assert_eq!(raw.count_at_value(TEST_VALUE_LEVEL * 3.0 / 4.0), 0);
    assert_eq!(raw.count_at_value(TEST_VALUE_LEVEL * 4.0 / 4.0), 1);
    assert_eq!(raw.total_count(), 2);
    
    let mut cor = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(cor.record_value(0.0));
    assert!(cor.record_corrected_value(TEST_VALUE_LEVEL, TEST_VALUE_LEVEL / 4.0));

    assert_eq!(cor.count_at_value(0.0), 1);
    assert_eq!(cor.count_at_value(TEST_VALUE_LEVEL * 1.0 / 4.0), 1);
    assert_eq!(cor.count_at_value(TEST_VALUE_LEVEL * 2.0 / 4.0), 1);
    assert_eq!(cor.count_at_value(TEST_VALUE_LEVEL * 3.0 / 4.0), 1);
    assert_eq!(cor.count_at_value(TEST_VALUE_LEVEL * 4.0 / 4.0), 1);    
    assert_eq!(cor.total_count(), 5);
}

#[test]
fn test_reset() {
    let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(TEST_VALUE_LEVEL));
    assert_eq!(h.count_at_value(TEST_VALUE_LEVEL), 1);
    assert_eq!(h.total_count(), 1);

    h.reset();

    assert_eq!(h.count_at_value(TEST_VALUE_LEVEL), 0);
    assert_eq!(h.total_count(), 0);
}

#[test]
fn test_add() {
    let mut this = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();
    let mut that = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    this.record_value(TEST_VALUE_LEVEL);
    this.record_value(TEST_VALUE_LEVEL * 1000.0);

    that.record_value(TEST_VALUE_LEVEL);
    that.record_value(TEST_VALUE_LEVEL * 1000.0);

    assert_eq!(this.add(&that), 0);

    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL), 2);
    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL * 1000.0), 2);
    assert_eq!(this.total_count(), 4);

    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL), 1);
    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL * 1000.0), 1);
    assert_eq!(that.total_count(), 2);
}

#[test]
fn test_add_smaller_to_bigger() {
    let mut this = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE * 2, SIGNIFICANT_FIGURES).unwrap();
    let mut that = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    this.record_value(TEST_VALUE_LEVEL);
    this.record_value(TEST_VALUE_LEVEL * 1000.0);
    
    that.record_value(TEST_VALUE_LEVEL);
    that.record_value(TEST_VALUE_LEVEL * 1000.0);

    assert_eq!(this.add(&that), 0);

    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL), 2);
    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL * 1000.0), 2);
    assert_eq!(this.total_count(), 4);

    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL), 1);
    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL * 1000.0), 1);
    assert_eq!(that.total_count(), 2);    
}

#[test]
fn test_add_bigger_to_smaller_out_of_range() {
    let mut this = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();
    let mut that = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE * 2, SIGNIFICANT_FIGURES).unwrap();

    this.record_value(TEST_VALUE_LEVEL);
    this.record_value(TEST_VALUE_LEVEL * 1000.0);
    this.record_value(1.0);

    that.record_value(TEST_VALUE_LEVEL);
    that.record_value(TEST_VALUE_LEVEL * 1000.0);
    that.record_value(1.0);

    assert_eq!(this.add(&that), 0);

    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL), 2);
    assert_eq!(this.count_at_value(TEST_VALUE_LEVEL * 1000.0), 2);
    assert_eq!(this.total_count(), 6);

    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL), 1);
    assert_eq!(that.count_at_value(TEST_VALUE_LEVEL * 1000.0), 1);
    assert_eq!(that.total_count(), 3);    
}

#[test]
fn test_size_of_equivalent_range() {
   let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(1.0));

    assert!(compare_double(1.0 / 1024.0, h.size_of_equivalent_value_range(1.0), 0.001));
    assert!(compare_double(2.0, h.size_of_equivalent_value_range(2500.0), 0.001));
    assert!(compare_double(4.0, h.size_of_equivalent_value_range(8191.0), 0.001));
    assert!(compare_double(8.0, h.size_of_equivalent_value_range(8192.0), 0.001));
    assert!(compare_double(8.0, h.size_of_equivalent_value_range(10000.0), 0.001));
}

#[test]
fn test_lowest_equivalent_value() {
   let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(1.0));

    assert!(compare_double(10000.0, h.lowest_equivalent_value(10007.0), 0.001));
    assert!(compare_double(10008.0, h.lowest_equivalent_value(10009.0), 0.001));
}

#[test]
fn test_highest_equivalent_value() {
   let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(1.0));

    assert!(compare_double(8183.99999, h.highest_equivalent_value(8180.0), 0.001));
    assert!(compare_double(8191.99999, h.highest_equivalent_value(8191.0), 0.001));
    assert!(compare_double(8199.99999, h.highest_equivalent_value(8193.0), 0.001));
    assert!(compare_double(9999.99999, h.highest_equivalent_value(9995.0), 0.001));
    assert!(compare_double(10007.99999, h.highest_equivalent_value(10007.0), 0.001));
    assert!(compare_double(10015.99999, h.highest_equivalent_value(10008.0), 0.001));
}

#[test]
fn test_median_equivalent_value() {
   let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(1.0));

    assert!(compare_double(4.002, h.median_equivalent_value(4.0), 0.001));
    assert!(compare_double(5.002, h.median_equivalent_value(5.0), 0.001));
    assert!(compare_double(4001.0, h.median_equivalent_value(4000.0), 0.001));
    assert!(compare_double(8002.0, h.median_equivalent_value(8000.0), 0.001));
    assert!(compare_double(10004.0, h.median_equivalent_value(10007.0), 0.001));
}

#[test]
fn test_clone() {
   let mut h = F64Histogram::init(TRACKABLE_VALUE_RANGE_SIZE, SIGNIFICANT_FIGURES).unwrap();

    assert!(h.record_value(1.0));

    assert_eq!(h.total_count(), 1);
    assert_eq!(h.count_at_value(1.0), 1);

    let b = h.clone();

    assert_eq!(h.total_count(), b.total_count());
    assert_eq!(h.count_at_value(1.0), b.count_at_value(1.0));
}
