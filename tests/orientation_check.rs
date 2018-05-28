#![feature(custom_attribute)]

extern crate spectrophoner;
extern crate image;
#[macro_use(array)]
#[macro_use(s)]
extern crate ndarray;

use std::path::Path;

use image::imageops::colorops;
use image::{GenericImage, ImageBuffer, Rgb, RgbImage, SubImage};
use ndarray::prelude::*;

use spectrophoner::img_dispatcher;


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



#[test]
fn orientation_check() {
    let path = Path::new("tests/resources/orientation_check.bmp");
    let mut img = image::open(path).unwrap().to_rgb();
    let width = img.width();
    let height = img.height();
    let img_slice = img.sub_image(0, 0, width, height);
    let extracted_layer = img_dispatcher::naive_layer_extractor(&img_slice);

    #[rustfmt_skip]
    let expected_layer_array = array![
        [0, 0, 0],
        [255, 0, 0],
        [0, 0, 255]
    ];

    assert_img_data_eq_by_element(extracted_layer.view(), expected_layer_array.view());
}
