use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use img_dispatcher::{ChannelExporter, ImgLayerId, StaticImgDispatcher, ImgLayerMetadata, ImgPacket};
use img_interpreter::{ImgInterpreter, SectionInterpreter, SectionInterpreterGenerator};
use mixer;
use mixer::Chunk;
use synth::{Oscillator, Waveform};
use util::PosF32;
use audio;

const SAMPLE_RATE: u32 = 44100;

/// hacky testing for now
pub fn conduct() {
    let chunk_width = 1;
    let img_path = Path::new("resources/ascending_line.png");
    let (mut img_dispatcher, channel_exporters) = StaticImgDispatcher::new(img_path, chunk_width);

    let mut interpreter_sample_receivers = Vec::<Receiver<Chunk>>::new();

    for channel_exporter in channel_exporters {
        let layers_metadata = channel_exporter.layers_metadata;
        let img_layers_receiver = channel_exporter.receiver;
        let (samples_sender, samples_receiver) = channel::<Vec<f32>>();
        interpreter_sample_receivers.push(samples_receiver);
        let mut interpreter =
            derive_img_interpreter(layers_metadata, img_layers_receiver, samples_sender);
        thread::spawn(move || {
            interpreter.interpret();
        });
    }

    thread::spawn(move || {
        img_dispatcher.begin_dispatch();
    });


    let mixed_samples_receiver = mixer::mix(interpreter_sample_receivers, 1000.);

    audio::stream_to_device(mixed_samples_receiver);
}

fn derive_img_interpreter(
    layers_metadata: Vec<ImgLayerMetadata>,
    img_layers_receiver: Receiver<ImgPacket>,
    samples_sender: Sender<Vec<f32>>,
) -> ImgInterpreter {
    const TEMP_HARDCODED_SAMPLES_PER_IMG_CHUNK: usize = 44100;

    let section_interpreter_generators =
        derive_simple_section_interpreter_generators(&layers_metadata);

    ImgInterpreter::new(
        layers_metadata,
        img_layers_receiver,
        samples_sender,
        TEMP_HARDCODED_SAMPLES_PER_IMG_CHUNK,
        section_interpreter_generators,
    )
}

fn naive_section_interpreter_generator(
    y_pos_ratio: f32,
    height_ratio: f32,
    total_img_height: usize,
) -> Vec<SectionInterpreter> {
    let mut section_interpreters = Vec::<SectionInterpreter>::new();

    let sections = 1000;

    for i in 0..sections {
        let offset = (i as f32) / (sections as f32);
        let offset_y_pos_ratio = y_pos_ratio + offset;
        let offset_height_ratio = height_ratio + offset;
        let freq = offset_height_ratio * 220.;
        let oscillator = Oscillator::new(
            Waveform::Sine,
            PosF32::new(freq),
            SAMPLE_RATE,
        );
        println!("oscillator freq: {:?}", freq);
        let y_start = clamp(
            (offset_y_pos_ratio * (total_img_height as f32)) as usize,
            0,
            total_img_height,
        );
        let y_end = clamp(
            ((offset_y_pos_ratio + offset_height_ratio) * (total_img_height as f32)) as usize,
            0,
            total_img_height,
        );
        section_interpreters.push(SectionInterpreter {
            oscillator,
            y_start,
            y_end,
        });
    }

    section_interpreters
}



fn derive_simple_section_interpreter_generators(
    layers_metadata: &Vec<ImgLayerMetadata>,
) -> HashMap<ImgLayerId, SectionInterpreterGenerator> {
    let mut generators: HashMap<ImgLayerId, SectionInterpreterGenerator> = HashMap::new();
    for metadata in layers_metadata {
        generators.insert(metadata.img_layer_id, naive_section_interpreter_generator);
    }
    generators
}

fn clamp<T>(val: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if val < min {
        return min;
    }
    if val > max {
        return max;
    }
    return val;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_below_min() {
        assert_eq!(clamp(-1, 0, 10), 0);
    }

    #[test]
    fn test_clamp_above_max() {
        assert_eq!(clamp(11, 0, 10), 10);
    }

    #[test]
    fn test_clamp_within_bounds() {
        assert_eq!(clamp(5, 0, 10), 5);
    }
}
