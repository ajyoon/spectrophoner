use std::collections::{HashMap};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

pub type Chunk = Vec<f32>;

const MIX_CHUNK_LEN: usize = 1024;
const THREAD_SLEEP_DUR: Duration = Duration::from_millis(10);

fn build_processed_by_receiver_map(receiver_ids: Vec<u16>) -> HashMap<u16, usize> {
    receiver_ids.iter().map(|id| (*id, 0)).collect()
}

#[inline]
pub fn add_samples(samples_being_added_to: &mut [f32], samples_being_added: &[f32]) {
    debug_assert!(samples_being_added.len() <= samples_being_added_to.len());
    for (i, sample) in samples_being_added.iter().enumerate() {
        samples_being_added_to[i] += sample;
    }
}

#[inline]
fn compress(uncompressed_samples: &[f32], expected_max_amp: f32) -> Vec<f32> {
    uncompressed_samples
        .iter()
        .map(|sample| sample / expected_max_amp)
        .collect::<Vec<f32>>()
}

pub fn mix(receivers: Vec<Receiver<Chunk>>, expected_max_amp: f32) -> Receiver<Vec<f32>> {
    let (mixed_chunk_sender, mixed_chunk_receiver) = channel::<Vec<f32>>();

    thread::spawn(move || {
        loop {
            let mut combined_samples: Vec<f32> = vec![0f32; MIX_CHUNK_LEN];
            for receiver in &receivers {
                let chunk = receiver.recv().unwrap();
                debug_assert_eq!(chunk.len(), MIX_CHUNK_LEN);
                add_samples(combined_samples.as_mut_slice(), chunk.as_slice());
                mixed_chunk_sender.send(
                    compress(combined_samples.as_slice(), expected_max_amp)
                ).unwrap();
            }
        }
    });

    mixed_chunk_receiver
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_add_samples() {
        let mut vec_being_added_to = vec![1.1, 2.2];
        let vec_being_added = vec![1., 2.];
        add_samples(vec_being_added_to.as_mut_slice(), vec_being_added.as_slice());
        assert_eq!(vec_being_added_to, vec![2.1, 4.2]);
    }

    #[test]
    fn test_compress() {
        let compressed_samples = compress(
            vec![-100., 0., 100.].as_slice(),
            100.
        );

        assert_array_almost_eq_by_element(compressed_samples, vec![-1., 0., 1.,]);
    }
}
