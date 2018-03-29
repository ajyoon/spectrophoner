use std::collections::{HashMap, BinaryHeap};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use ::chunk::{Chunk};





fn mix(receivers: HashMap<u16, Receiver<Chunk>>) -> Receiver<Chunk> {

    let chunks = Arc::new(Mutex::new(BinaryHeap::new()));

    thread::spawn(move || {
        let chunks = Arc::clone(&chunks);
        loop {
            // receive chunks
            for receiver in receivers.values() {
                if let Ok(chunk) = receiver.try_recv() {
                    chunks.lock().unwrap().push(chunk);
                }
            }
        }
    });

    let (mixed_chunk_sender, mixed_chunk_receiver) = channel::<Chunk>();

    thread::spawn(move || {
        let chunks = Arc::clone(&chunks);
        let processed_by_sender = HashMap::<u16, u32>::new();
        for sender_id in receivers.keys() {
            processed_by_sender.insert(*sender_id, 0);
        }

        loop {

        }
    });

    mixed_chunk_receiver;
}
