use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use img_dispatcher::{ChannelExporter, ImgLayerId, StaticImgDispatcher, ImgLayerMetadata, ImgPacket};
use img_interpreter::{ImgInterpreter, SectionInterpreter, SectionInterpreterGenerator};
use synth::{Oscillator, Waveform};
use util::PosF32;

const SAMPLE_RATE: u32 = 44100;

/// hacky testing for now
pub fn conduct() {
    let chunk_width = 10;
    let img_path = Path::new("../resources/ascending_line.png");
    let (mut img_dispatcher, channel_exporters) = StaticImgDispatcher::new(img_path, chunk_width);

    for channel_exporter in channel_exporters {
        let layers_metadata = channel_exporter.layers_metadata;
        let img_layers_receiver = channel_exporter.receiver;
        let (samples_sender, samples_receiver) = channel::<Vec<f32>>();
        let interpreter =
            derive_img_interpreter(layers_metadata, img_layers_receiver, samples_sender);
    }

    thread::spawn(move || {
        img_dispatcher.begin_dispatch();
    });


    ()
}

fn derive_img_interpreter(
    layers_metadata: Vec<ImgLayerMetadata>,
    img_layers_receiver: Receiver<ImgPacket>,
    samples_sender: Sender<Vec<f32>>,
) -> ImgInterpreter {
    const TEMP_HARDCODED_SAMPLES_PER_IMG_CHUNK: usize = 441000;

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
    let oscillator = Oscillator::new(
        Waveform::Sine,
        PosF32::new(height_ratio * 440.),
        SAMPLE_RATE,
    );
    let y_start = clamp(
        (y_pos_ratio * (total_img_height as f32)) as usize,
        0,
        total_img_height,
    );
    let y_end = clamp(
        ((y_pos_ratio + height_ratio) * (total_img_height as f32)) as usize,
        0,
        total_img_height,
    );
    vec![SectionInterpreter {
        oscillator,
        y_start,
        y_end,
    }]
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
