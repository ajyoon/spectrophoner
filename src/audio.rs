use std::sync::mpsc::{channel, Receiver};
use std::thread;
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

    let mut queued_received_samples: Vec<f32> = Vec::new();

    let callback = move |args: portaudio::OutputStreamCallbackArgs<f32>| {
        let mut idx = 0;
        let received_samples = chunk_receiver.try_recv().unwrap();
        for _ in 0..args.frames {
            //buffer[idx]
            // buffer[idx]   = sine[left_phase];
            // buffer[idx+1] = sine[right_phase];
            // left_phase += 1;
            // if left_phase >= TABLE_SIZE { left_phase -= TABLE_SIZE; }
            // right_phase += 3;
            // if right_phase >= TABLE_SIZE { right_phase -= TABLE_SIZE; }
            idx += 2;
        }
        portaudio::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();

    stream.start().unwrap();
}
