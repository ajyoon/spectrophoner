extern crate libc;

use std::mem;

use ndarray::prelude::*;


#[inline]
fn unsafe_memcpy(write_ptr: usize, src_ptr: usize, bytes: usize) {
    unsafe {
        libc::memcpy(
            write_ptr as *mut libc::c_void,
            src_ptr as *mut libc::c_void,
            bytes,
        );
    }
}


pub fn roll_vec<T>(src_vec: &Vec<T>, src_offset: usize, elements: usize) -> Vec<T> {
    let element_size = mem::size_of::<T>();
    let roll_offset = src_offset % src_vec.len();
    let mut rolled = Vec::<T>::with_capacity(elements);

    let mut src_ptr;
    let mut write_ptr = rolled.as_slice().as_ptr() as usize;

    let mut copied_elements = 0;

    if roll_offset != 0 {
        src_ptr = src_vec.as_slice().split_at(roll_offset).1.as_ptr() as usize;
        let bytes = (src_vec.len() - roll_offset) * element_size;
        unsafe_memcpy(write_ptr, src_ptr, bytes);
        write_ptr = write_ptr + bytes;
        copied_elements = src_vec.len() - roll_offset;
    }

    let full_rolls = (elements - copied_elements) / src_vec.len();
    let bytes_per_copy = src_vec.len() * element_size;
    src_ptr = src_vec.as_slice().as_ptr() as usize;
    for _ in 0..full_rolls {
        unsafe_memcpy(write_ptr, src_ptr, bytes_per_copy);
        write_ptr = write_ptr + bytes_per_copy;
    }
    copied_elements += full_rolls * src_vec.len();

    let remaining_elements = elements - copied_elements;
    if remaining_elements > 0 {
        src_ptr = src_vec.as_slice().split_at(remaining_elements).0.as_ptr() as usize;
        let bytes = remaining_elements * element_size;
        unsafe_memcpy(write_ptr, src_ptr, bytes);
    }

    unsafe {
        rolled.set_len(elements);
    }

    rolled
}

pub fn multiply_over_linspace(data: &mut[f32], start: f32, end: f32) {
    let multipliers = Array::linspace(start, end, data.len());
    for (i, element) in data.iter_mut().enumerate() {
        *element *= multipliers[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_utils::*;

    #[test]
    fn single_complete_copy() {
        let original = vec![1, 2, 3];
        let rolled = roll_vec(&original, 0, 3);
        assert_eq!(rolled, original);
    }

    #[test]
    fn with_head() {
        let original = vec![1, 2, 3];
        let rolled = roll_vec(&original, 2, 4);
        assert_eq!(rolled, [3, 1, 2, 3]);
    }

    #[test]
    fn with_head_body_and_tail() {
        let original = vec![1, 2, 3];
        let rolled = roll_vec(&original, 1, 4);
        assert_eq!(rolled, [2, 3, 1, 2]);
    }

    #[test]
    fn with_head_body_and_tail_multiple_bodies() {
        let original = vec![1, 2, 3];
        let rolled = roll_vec(&original, 1, 9);
        assert_eq!(rolled, [2, 3, 1, 2, 3, 1, 2, 3, 1]);
    }

    #[test]
    fn with_head_body_and_tail_multiple_bodies_different_dtype() {
        let original = vec![1., 2., 3.];
        let rolled = roll_vec(&original, 1, 9);
        assert_eq!(rolled, [2., 3., 1., 2., 3., 1., 2., 3., 1.]);
    }

    #[test]
    fn test_multiply_over_linespace_constant() {
        let mut vec = vec![1., 2., 3.];
        multiply_over_linspace(vec.as_mut_slice(), 1., 1.);
        assert_almost_eq_by_element(vec, vec![1., 2., 3.]);
    }

    #[test]
    fn test_multiply_over_linespace_slope_1() {
        let mut vec = vec![1., 2., 3.];
        multiply_over_linspace(vec.as_mut_slice(), 0., 1.);
        assert_almost_eq_by_element(vec, vec![0., 1., 3.]);
    }
}
