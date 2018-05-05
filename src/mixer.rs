use std::collections::{BinaryHeap, HashMap};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::cmp;

use chunk::Chunk;

const MIX_CHUNK_LEN: usize = 1024;
const THREAD_SLEEP_DUR: Duration = Duration::from_millis(10);

// Infinitely loop and receive chunks from a map of receivers,
// placing their chunks into a priority queue.
fn collect_chunks(
    receivers: HashMap<u16, Receiver<Chunk>>,
    collection_queue: Arc<Mutex<BinaryHeap<Chunk>>>,
) {
    loop {
        // receive chunks
        for receiver in receivers.values() {
            if let Ok(chunk) = receiver.try_recv() {
                collection_queue.lock().unwrap().push(chunk);
            }
        }
    }
}

fn build_processed_by_sender_map(sender_ids: Vec<u16>) -> HashMap<u16, usize> {
    sender_ids.iter().map(|id| (*id, 0)).collect()
}

fn mix_chunks<'a>(
    chunks_queue: Arc<Mutex<BinaryHeap<Chunk>>>,
    uncompressed_chunk_sender: Sender<Vec<f32>>,
    sender_ids: Vec<u16>,
) {
    let mut processed_by_sender = build_processed_by_sender_map(sender_ids);
    let mut current_chunk_start = 0;
    let mut current_chunk_end = current_chunk_start + MIX_CHUNK_LEN;
    let mut combined_chunk: Vec<f32> = vec![0.0; MIX_CHUNK_LEN];
    loop {
        let mut locked_chunks_queue = chunks_queue.lock().unwrap();
        let mut maybe_chunk = locked_chunks_queue.pop();
        match maybe_chunk {
            Some(mut chunk) => {
                if chunk.start_sample > current_chunk_end {
                    // Found a chunk, but it is past the current chunk.
                    // Place back in the queue, drop queue lock, and sleep
                    locked_chunks_queue.push(chunk);
                    drop(locked_chunks_queue);
                    thread::sleep(THREAD_SLEEP_DUR);
                    continue;
                }

                // Drop queue lock while processing
                drop(locked_chunks_queue);

                // Add as many samples as possible to `combined_chunk`
                let samples_to_take_from_chunk =
                    cmp::min(current_chunk_end - chunk.start_sample, chunk.samples.len());
                let offset_in_combined_chunk = chunk.start_sample - current_chunk_start;
                for (i, sample) in chunk
                    .samples
                    .drain(..samples_to_take_from_chunk)
                    .enumerate()
                {
                    combined_chunk[i + offset_in_combined_chunk] += sample;
                }

                // Push any left-over samples back onto to queue.
                if samples_to_take_from_chunk < chunk.samples.len() {
                    chunks_queue.lock().unwrap().push(Chunk {
                        start_sample: chunk.start_sample + samples_to_take_from_chunk,
                        sender_id: chunk.sender_id,
                        samples: chunk.samples,
                    });
                }

                // Mark samples as processed for the sender id
                *processed_by_sender.get_mut(&chunk.sender_id).unwrap() +=
                    samples_to_take_from_chunk;

                // If all senders have been processed for the current chunk, ship it!
                if processed_by_sender
                    .values()
                    .all(|processed_until| processed_until >= &current_chunk_end)
                {
                    uncompressed_chunk_sender.send(combined_chunk);
                    combined_chunk = vec![0.0; MIX_CHUNK_LEN];
                    current_chunk_start = current_chunk_end;
                    current_chunk_end = current_chunk_start + MIX_CHUNK_LEN;
                }
            }
            None => {
                // Found no chunk. Drop queue lock and sleep.
                drop(locked_chunks_queue);
                thread::sleep(THREAD_SLEEP_DUR);
            }
        }
    }
}

fn compress(
    uncompressed_chunk_receiver: Receiver<Vec<f32>>,
    compressed_chunk_sender: Sender<Vec<i16>>,
    expected_max_amp: f32,
) {
    for chunk in uncompressed_chunk_receiver {
        compressed_chunk_sender
            .send(
                chunk
                    .iter()
                    .map(|sample| ((sample / expected_max_amp) * 32767.) as i16)
                    .collect::<Vec<i16>>(),
            )
            .unwrap();
    }
}

pub fn mix(receivers: HashMap<u16, Receiver<Chunk>>, expected_max_amp: f32) -> Receiver<Vec<i16>> {
    let chunks_queue = Arc::new(Mutex::new(BinaryHeap::new()));
    let receiver_ids = receivers.keys().map(|key| *key).collect::<Vec<u16>>();
    let chunks_queue_pusher_ref = Arc::clone(&chunks_queue);
    thread::spawn(move || {
        collect_chunks(receivers, chunks_queue_pusher_ref);
    });

    // Thread for pulling chunks off the `chunks_queue` priority queue
    // and mixing them, sending uncompressed chunks through a channel
    let (uncompressed_chunk_sender, uncompressed_chunk_receiver) = channel::<Vec<f32>>();
    thread::spawn(move || {
        mix_chunks(chunks_queue, uncompressed_chunk_sender, receiver_ids);
    });

    let (mixed_chunk_sender, mixed_chunk_receiver) = channel::<Vec<i16>>();
    thread::spawn(move || {
        compress(
            uncompressed_chunk_receiver,
            mixed_chunk_sender,
            expected_max_amp,
        );
    });

    mixed_chunk_receiver
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::{thread_rng, Rng};

    #[test]
    fn test_collect_chunks() {
        // Set up receivers
        let mut receivers: HashMap<u16, Receiver<Chunk>> = HashMap::new();

        let (sender_0, receiver_0) = channel::<Chunk>();
        let (sender_1, receiver_1) = channel::<Chunk>();
        receivers.insert(0, receiver_0);
        receivers.insert(1, receiver_1);

        // Set up chunks_queue
        let chunks_queue = Arc::new(Mutex::new(BinaryHeap::new()));
        //let receiver_ids = receivers.keys().map(|key| *key).collect::<Vec<u16>>();

        let chunks_queue_pusher_ref = Arc::clone(&chunks_queue);
        thread::spawn(move || {
            collect_chunks(receivers, chunks_queue_pusher_ref);
        });

        let later_chunk_from_sender_0 = chunk_with_unique_samples(100, 0);
        let earlier_chunk_from_sender_1 = chunk_with_unique_samples(0, 1);

        sender_0.send(later_chunk_from_sender_0).unwrap();
        sender_1.send(earlier_chunk_from_sender_1).unwrap();

        wait_a_smidge();  // so collector has time to process the chunks

        assert_eq!(chunks_queue.lock().unwrap().pop().unwrap().start_sample, 0);
        assert_eq!(chunks_queue.lock().unwrap().pop().unwrap().start_sample, 100);
    }

    #[test]
    fn test_compress() {
        let (uncompressed_chunk_sender, uncompressed_chunk_receiver) = channel::<Vec<f32>>();
        let (mixed_chunk_sender, mixed_chunk_receiver) = channel::<Vec<i16>>();

        uncompressed_chunk_sender.send(vec![-100., 0.]).unwrap();
        uncompressed_chunk_sender.send(vec![100., 0.]).unwrap();
        drop(uncompressed_chunk_sender);

        compress(
            uncompressed_chunk_receiver,
            mixed_chunk_sender,
            100.,
        );

        let first_mixed_chunk = mixed_chunk_receiver.recv().unwrap();
        let last_mixed_chunk = mixed_chunk_receiver.recv().unwrap();

        assert_eq!(first_mixed_chunk, vec![-32767, 0]);
        assert_eq!(last_mixed_chunk, vec![32767, 0]);
    }

    /// Generate a test chunk with 3 randomly generated samples
    fn chunk_with_unique_samples(start_sample: usize, sender_id: u16) -> Chunk {
        let mut rng = thread_rng();
        let mut samples = Vec::<f32>::new();
        for _ in 0..3 {
            samples.push(rng.gen());
        }
        Chunk {
            start_sample,
            sender_id,
            samples
        }
    }

    fn wait_a_smidge() {
        thread::sleep(Duration::from_millis(25));
    }
}
