use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use portaudio;
use stopwatch::Stopwatch;

use sample_buffer::SampleBuffer;

const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 1024;
const THREAD_SLEEP_DUR: Duration = Duration::from_millis(500);

pub fn stream_to_device(chunk_receiver: Receiver<Vec<f32>>) {
    let pa = portaudio::PortAudio::new().unwrap();
    let mut settings = pa
        .default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)
        .unwrap();

    let initial_queue_buffer = SampleBuffer::new(Vec::<f32>::new());
    let queued_received_samples = Arc::new(RefCell::new(initial_queue_buffer));

    let callback = move |args: portaudio::OutputStreamCallbackArgs<f32>| {
        let mut queued_samples = queued_received_samples.borrow_mut();
        let mut buffer_index = 0;
        loop {
            let queued_elements_remaining = queued_samples.elements_remaining();
            if queued_elements_remaining >= args.buffer.len() {
                // Enough samples in the queue to fill the buffer completely
                queued_samples.consume_into(&args.buffer);
                return portaudio::Continue;
            }

            // Not enough samples in the queue to fill the buffer, but take what we can
            queued_samples
                .consume_into(&args.buffer[buffer_index..buffer_index + queued_elements_remaining]);

            // Fill the queue with a new received chunk
            let sw = Stopwatch::start_new();
            let received_samples = chunk_receiver.recv().unwrap();
            println!("received samples in {:?}", sw.elapsed());
            queued_samples.overwrite(received_samples);
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();

    stream.start().unwrap();

    loop {
        println!("[heartbeat]");
        thread::sleep(THREAD_SLEEP_DUR);
    }
}
