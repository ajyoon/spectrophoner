extern crate libc;

use std::mem;


pub struct SampleBuffer<T> {
    underlier: Vec<T>,
    elem_size: usize,
    index: usize,
}


impl <T> SampleBuffer<T> {
    pub fn new(source: Vec<T>) -> SampleBuffer<T> {
        SampleBuffer {
            underlier: source,
            elem_size: mem::size_of::<T>(),
            index: 0,
        }
    }

    #[inline]
    fn reset_index(&mut self) {
        self.index = 0;
    }

    #[inline]
    pub fn elements_remaining(&self) -> usize {
        self.underlier.len() - self.index
    }

    pub fn overwrite(&mut self, new_data: Vec<T>) {
        self.underlier = new_data;
        self.reset_index();
    }

    /// fill `target` with consumed samples from the buffer
    /// `target.len()` must be less than or equal to `self.elements_remaining()`
    pub fn consume_into(&mut self, target: &[T]) {
        let element_count = target.len();
        debug_assert!(element_count <= self.elements_remaining());
        let src_ptr = (&self.underlier[self.index..]).as_ptr() as *mut libc::c_void;
        let write_ptr = target.as_ptr() as *mut libc::c_void;
        let bytes = self.elem_size * element_count;
        unsafe {
            libc::memcpy(
                write_ptr,
                src_ptr,
                bytes
            );
        }
        self.index += element_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_init() {
        let buf = SampleBuffer::new(Vec::<f32>::new());
        assert_eq!(buf.underlier, Vec::<f32>::new());
        assert_eq!(buf.elem_size, 4);
        assert_eq!(buf.index, 0);
    }

    #[test]
    fn test_overwrite() {
        let mut buf = SampleBuffer::new(Vec::<f32>::new());
        let new_data = vec![0.1, 0.2];
        buf.overwrite(new_data);

        assert_almost_eq_by_element(buf.underlier, vec![0.1, 0.2]);
        assert_eq!(buf.index, 0);
    }

    #[test]
    fn test_consume_into_all_samples_available() {
        let mut buf = SampleBuffer::<u8>::new(vec![8, 9]);
        let target = vec![1, 2, 3];
        buf.consume_into(&target[..2]);
        assert_eq!(target, vec![8, 9, 3]);
    }

    #[test]
    fn test_consume_into_repeatedly() {
        let mut buf = SampleBuffer::<u8>::new(vec![1, 2, 3, 4, 5]);
        let target = vec![0; 5];
        buf.consume_into(&target[..3]);
        assert_eq!(target, vec![1, 2, 3, 0, 0]);
        buf.consume_into(&target[3..]);
        assert_eq!(target, vec![1, 2, 3, 4, 5]);
    }
}
