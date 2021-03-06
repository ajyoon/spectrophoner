use ndarray::prelude::*;

pub fn assert_almost_eq_by_element(left: Vec<f32>, right: Vec<f32>) {
    const F32_EPSILON: f32 = 1.0e-6;
    if left.len() != right.len() {
        panic!(
            "lengths differ: left.len() = {}, right.len() = {}",
            left.len(),
            right.len()
        );
    }
    for (left_val, right_val) in left.iter().zip(right.iter()) {
        assert!(
            (*left_val - *right_val).abs() < F32_EPSILON,
            "{} is not approximately equal to {}. \
             complete left vec: {:?}. complete right vec: {:?}",
            *left_val,
            *right_val,
            left,
            right
        );
    }
}

pub fn assert_img_data_eq_by_element(left: ArrayView2<u8>, right: ArrayView2<u8>) {
    if left.len() != right.len() {
        panic!(
            "lengths differ: left.len() = {}, right.len() = {}",
            left.len(),
            right.len()
        );
    }
    for (left_val, right_val) in left.iter().zip(right.iter()) {
        assert!(
            left_val == right_val,
            "{} is not approximately equal to {}. \
             complete left array: \n{:?} \n \
             complete right array: \n{:?} \n",
            *left_val,
            *right_val,
            left,
            right
        );
    }
}

pub fn assert_almost_eq(left: f32, right: f32) {
    const F32_EPSILON: f32 = 1.0e-4;
    assert!(
        (left - right).abs() < F32_EPSILON,
        "{} is not approximately equal to {}.",
        left,
        right,
    );
}
