use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use portaudio;
use stopwatch::Stopwatch;

use audio_streamer::AudioStreamer;
use sample_buffer::SampleBuffer;

const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 1024;
const THREAD_SLEEP_DUR: Duration = Duration::from_millis(500);

pub struct PortAudioStreamer<T> {
    portaudio: portaudio::PortAudio,
    portaudio_settings: portaudio::OutputStreamSettings<T>,
}

impl<T> PortAudioStreamer<T> {
    pub fn new() -> PortAudioStreamer<T> {
        let pa = portaudio::PortAudio::new().unwrap();
        let settings = pa
            .default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)
            .unwrap();
        PortAudioStreamer {
            portaudio: pa,
            portaudio_settings: settings,
        }
    }
}

impl<T: 'static> AudioStreamer<T> for PortAudioStreamer<T>
where T: portaudio::Sample {
    fn stream(&self, chunk_receiver: Receiver<Vec<T>>) {
        let initial_queue_buffer = SampleBuffer::new(Vec::<T>::new());
        let queued_received_samples = Arc::new(RefCell::new(initial_queue_buffer));

        let callback = move |args: portaudio::OutputStreamCallbackArgs<T>| {
            fill_pa_buffer(&queued_received_samples, &chunk_receiver, args.buffer);
            portaudio::Continue
        };

        let mut stream = self
            .portaudio
            .open_non_blocking_stream(self.portaudio_settings, callback)
            .unwrap();
        stream.start().unwrap();

        loop {
            thread::sleep(THREAD_SLEEP_DUR);
        }
    }
}

fn fill_pa_buffer<T>(
    queued_received_samples: &Arc<RefCell<SampleBuffer<T>>>,
    chunk_receiver: &Receiver<Vec<T>>,
    out_buffer: &mut [T],
) {
    let mut queued_samples = queued_received_samples.borrow_mut();
    let mut buffer_index = 0;
    loop {
        let queued_elements_remaining = queued_samples.elements_remaining();
        if queued_elements_remaining >= out_buffer.len() - buffer_index {
            // Enough samples in the queue to fill the buffer completely
            queued_samples.consume_into(&out_buffer[buffer_index..]);
            return;
        }

        // Not enough samples in the queue to fill the buffer, but take what we can
        queued_samples
            .consume_into(&out_buffer[buffer_index..buffer_index + queued_elements_remaining]);

        buffer_index += queued_elements_remaining;

        // Fill the queue with a new received chunk
        // let sw = Stopwatch::start_new();
        let received_samples = chunk_receiver.recv().unwrap();
        // println!("received samples in {:?}", sw.elapsed());
        queued_samples.overwrite(received_samples);
    }
}
