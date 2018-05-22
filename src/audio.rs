use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::time::Duration;

use portaudio;


const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const THREAD_SLEEP_DUR: Duration = Duration::from_millis(500);


// TODO update mixer.rs to not do i16 casting and just leave it f32

pub fn stream_to_device(chunk_receiver: Receiver<Vec<f32>>) {
    let pa = portaudio::PortAudio::new().unwrap();
    let mut settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER).unwrap();
    settings.flags = portaudio::stream_flags::CLIP_OFF;

    let initial_queue_vec = Vec::<f32>::new();
    let queued_received_samples = Arc::new(RefCell::new(initial_queue_vec));

    // TODO: optimize callback using memcpy
    let callback = move |args: portaudio::OutputStreamCallbackArgs<f32>| {

        let mut queued_samples = queued_received_samples.borrow_mut();

        let mut buffer_index = 0;
        if !(*queued_samples).is_empty() {
            if (*queued_samples).len() < args.buffer.len() {
                // Consume all remaining queued samples and continue
                for sample in (*queued_samples).drain(0..) {
                    args.buffer[buffer_index] = sample;
                    buffer_index += 1;
                }
            } else {
                // Consume as many remaining queued samples as possible,
                // filling the buffer and returning early
                for sample in (*queued_samples).drain(args.buffer.len()..) {
                    args.buffer[buffer_index] = sample;
                    buffer_index += 1;
                }
                return portaudio::Continue;
            }
        }

        loop {
            let mut received_samples = chunk_receiver.try_recv().unwrap();
            if received_samples.len() < args.buffer.len() - buffer_index {
                // Not enough received samples to fill buffer - consume all and repeat
                for sample in received_samples {
                    args.buffer[buffer_index] = sample;
                    buffer_index += 1;
                }
            } else {
                // Exactly enough or too many received samples for buffer,
                // consume everything we can and queue any left-overs
                for sample in received_samples.drain((args.buffer.len() - buffer_index)..) {
                    args.buffer[buffer_index] = sample;
                    buffer_index += 1;
                }
                if !received_samples.is_empty() {
                    let remaining_samples = received_samples.clone();
                    *queued_samples = remaining_samples;
                }
                return portaudio::Continue;
            }
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();

    stream.start().unwrap()
}
