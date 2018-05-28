use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

use stopwatch::Stopwatch;

pub type Chunk = Vec<f32>;

#[inline]
fn add_chunk_to_maybe_empty(src: &Chunk, dest: &mut Chunk) {
    debug_assert!(dest.is_empty() || dest.len() == src.len());
    if dest.is_empty() {
        dest.reserve_exact(src.len());
        for i in 0..src.len() {
            dest.push(0.);
        }
    }
    for (i, sample) in src.iter().enumerate() {
        unsafe {
            *dest.get_unchecked_mut(i) += sample;
        }
    }
}

#[inline]
pub fn add_chunk_to(src: &Chunk, dest: &mut Chunk) {
    debug_assert!(dest.len() == src.len());
    for (i, sample) in src.iter().enumerate() {
        unsafe {
            *dest.get_unchecked_mut(i) += sample;
        }
    }
}

#[inline]
fn compress(uncompressed_samples: &mut Vec<f32>, expected_max_amp: f32) {
    for sample in uncompressed_samples {
        *sample = (*sample) / expected_max_amp;
    }
}

pub fn mix(receivers: Vec<Receiver<Chunk>>, expected_max_amp: f32) -> Receiver<Vec<f32>> {
    let (mixed_chunk_sender, mixed_chunk_receiver) = channel::<Vec<f32>>();

    thread::Builder::new().name("mixer::mix()".to_string()).spawn(move || {
        loop {
            // let sw = Stopwatch::start_new();
            let mut combined_samples: Vec<f32> = Vec::new();
            for receiver in &receivers {
                let chunk = receiver.recv().unwrap();
                add_chunk_to_maybe_empty(&chunk, &mut combined_samples);
            }
            compress(&mut combined_samples, expected_max_amp);
            mixed_chunk_sender.send(combined_samples).unwrap();
            // println!("mixed samples in {:?}", sw.elapsed());
        }
    });

    mixed_chunk_receiver
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_add_chunk_to_maybe_empty_from_empty() {
        let mut dest = vec![];
        let src = vec![1., 2.];
        add_chunk_to_maybe_empty(&src, &mut dest);
        assert_almost_eq_by_element(dest, vec![1., 2.]);
    }

    #[test]
    fn test_add_chunk_to_from_same_size() {
        let mut dest = vec![1.1, 2.2];
        let src = vec![1., 2.];
        add_chunk_to(&src, &mut dest);
        assert_almost_eq_by_element(dest, vec![2.1, 4.2]);
    }

    #[test]
    fn test_compress() {
        let mut samples = vec![-100., 0., 100.];
        compress(&mut samples, 100.);

        assert_almost_eq_by_element(samples, vec![-1., 0., 1.,]);
    }
}

#[cfg(test)]
mod benchmarks {
    extern crate test;
    extern crate rand;
    use super::*;

    #[bench]
    fn add_chunk_to_maybe_empty_with_empty(b: &mut test::Bencher) {
        run_add_chunk_maybe_empty_bench(b, random_chunk(44100), vec![]);
    }

    #[bench]
    fn add_chunk_to_maybe_empty_with_nonempty(b: &mut test::Bencher) {
        run_add_chunk_maybe_empty_bench(b, random_chunk(44100), random_chunk(44100));
    }

    #[bench]
    fn add_chunk_to_filled(b: &mut test::Bencher) {
        run_add_chunk_bench(b, random_chunk(44100), random_chunk(44100));
    }

    #[bench]
    fn compress_random_data(b: &mut test::Bencher) {
        run_compress_bench(b, &mut random_chunk(44100));
    }

    fn run_add_chunk_bench(b: &mut test::Bencher, src: Vec<f32>, dest: Vec<f32>) {
        let mut black_box_dest = test::black_box(dest);
        b.iter(|| {
            add_chunk_to(&src, &mut black_box_dest);
        });
    }

    fn run_add_chunk_maybe_empty_bench(b: &mut test::Bencher, src: Vec<f32>, dest: Vec<f32>) {
        let mut black_box_dest = test::black_box(dest);
        b.iter(|| {
            add_chunk_to_maybe_empty(&src, &mut black_box_dest);
        });
    }

    fn run_compress_bench(b: &mut test::Bencher, samples: &mut Chunk) {
        b.iter(|| {
            compress(samples, 100.);
        });
    }

    fn random_chunk(len: usize) -> Vec<f32> {
        let mut chunk = Vec::<f32>::new();
        for _ in 0..len {
            chunk.push(rand::random::<f32>());
        }
        chunk
    }
}
