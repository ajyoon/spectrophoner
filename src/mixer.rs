use std::collections::{BinaryHeap, HashMap};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::cmp;

use chunk::Chunk;

const MIX_CHUNK_LEN: usize = 1024;

// Infinitely loop and receive chunks from a map of receivers,
// placing their chunks into a priority queue.
fn collect_chunks(receivers: HashMap<u16, Receiver<Chunk>>,
                  collection_queue: Arc<Mutex<BinaryHeap<Chunk>>>) {
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

fn mix_chunks<'a>(chunks_queue: Arc<Mutex<BinaryHeap<Chunk>>>,
              uncompressed_chunk_sender: Sender<Vec<f32>>,
              sender_ids: Vec<u16>) {
    let thread_sleep_dur = Duration::from_millis(10);
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
                    thread::sleep(thread_sleep_dur);
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
                thread::sleep(thread_sleep_dur);
            }
        }
    }
}

pub fn mix(receivers: HashMap<u16, Receiver<Chunk>>) -> Receiver<Vec<i16>> {
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
        for chunk in uncompressed_chunk_receiver {

        }
    });

    mixed_chunk_receiver
}
