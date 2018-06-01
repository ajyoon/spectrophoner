use std::marker::PhantomData;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::mem;

use hound;
use num;

use audio_streamer::AudioStreamer;
use sample_buffer::SampleBuffer;

const TEMP_HARDCODED_SAMPLE_RATE: u32 = 44100;

// currently only works with f32 samples

pub struct WavStreamer<T> {
    spec: hound::WavSpec,
    out_path: String,
    phantom: PhantomData<T>,
}

impl <T> WavStreamer<T> where T: num::Num {
    pub fn new(out_path: String) -> WavStreamer<T> {
        // For now, we use some hardcoded settings
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TEMP_HARDCODED_SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        WavStreamer {
            spec,
            out_path,
            phantom: PhantomData
        }
    }
}

impl<T: 'static> AudioStreamer<T> for WavStreamer<f32>
where T: num::ToPrimitive + num::Num {
    fn stream(&self, chunk_receiver: Receiver<Vec<T>>) {
        let mut writer = hound::WavWriter::create(&self.out_path, self.spec).unwrap();
        for chunk in chunk_receiver {
            for sample in chunk {
                writer.write_sample(sample.to_f32().unwrap()).unwrap();
            }
        }
    }
}
