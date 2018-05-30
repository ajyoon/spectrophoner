use std::cmp::Ordering::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use audio_streamer::AudioStreamer;
use portaudio_streamer::PortAudioStreamer;
use img_dispatcher::{
    ChannelExporter, ImgLayerId, ImgLayerMetadata, ImgPacket, StaticImgDispatcher,
};
use img_interpreter::{ImgInterpreter, SectionInterpreter};
use mixer;
use mixer::Chunk;
use pitch;
use synth::{Oscillator, Waveform};

const SAMPLE_RATE: u32 = 44100;

const TEMP_HARDCODED_SAMPLES_PER_PIXEL: usize = 4410;
const TEMP_HARDCODED_IMG_CHUNK_WIDTH: u32 = 100;
const TEMP_HARDCODED_OSC_COUNT: usize = 60;

/// hacky testing for now
pub fn conduct() {
    let img_path = Path::new("resources/ascending_line.png");
    let (mut img_dispatcher, channel_exporters) =
        StaticImgDispatcher::new(img_path, TEMP_HARDCODED_IMG_CHUNK_WIDTH);

    let mut interpreter_sample_receivers = Vec::<Receiver<Chunk>>::new();

    for channel_exporter in channel_exporters {
        let layers_metadata = channel_exporter.layers_metadata;
        let img_layers_receiver = channel_exporter.receiver;
        let (samples_sender, samples_receiver) = channel::<Vec<f32>>();
        interpreter_sample_receivers.push(samples_receiver);
        let mut interpreter =
            derive_img_interpreter(layers_metadata, img_layers_receiver, samples_sender);

        thread::Builder::new()
            .name("ImgInterpreter".to_string())
            .spawn(move || {
                interpreter.interpret();
            })
            .unwrap();
    }

    thread::Builder::new()
        .name("StaticImgDispatcher".to_string())
        .spawn(move || {
            img_dispatcher.begin_dispatch();
        })
        .unwrap();

    let mixed_samples_receiver = mixer::mix(
        interpreter_sample_receivers,
        (TEMP_HARDCODED_OSC_COUNT as f32) * 0.15,
    );

    PortAudioStreamer::new().stream(mixed_samples_receiver);
}

fn derive_img_interpreter(
    layers_metadata: Vec<ImgLayerMetadata>,
    img_layers_receiver: Receiver<ImgPacket>,
    samples_sender: Sender<Vec<f32>>,
) -> ImgInterpreter {
    let layer_handlers = derive_layer_handlers(layers_metadata);
    ImgInterpreter::new(
        img_layers_receiver,
        samples_sender,
        TEMP_HARDCODED_SAMPLES_PER_PIXEL,
        layer_handlers,
    )
}

fn derive_layer_handlers(
    layers_metadata: Vec<ImgLayerMetadata>,
) -> HashMap<ImgLayerId, Vec<SectionInterpreter>> {
    let mut layer_handlers = HashMap::new();
    for layer_metadata in layers_metadata {
        layer_handlers.insert(
            layer_metadata.img_layer_id,
            generate_naive_section_interpreters(layer_metadata),
        );
    }
    layer_handlers
}

fn generate_naive_section_interpreters(
    layer_metadata: ImgLayerMetadata,
) -> Vec<SectionInterpreter> {
    let mut section_interpreters = Vec::<SectionInterpreter>::new();

    let mut frequencies = pitch::harmonic_series(23.5, TEMP_HARDCODED_OSC_COUNT);
    frequencies.reverse();

    let section_height = (layer_metadata.y_end - layer_metadata.y_start) / TEMP_HARDCODED_OSC_COUNT;

    for i in 0..TEMP_HARDCODED_OSC_COUNT {
        let y_start = clamp(
            layer_metadata.y_start + (section_height * i),
            layer_metadata.y_start,
            layer_metadata.y_end,
        );
        let y_end = clamp(
            layer_metadata.y_start + (section_height * (i + 1)),
            layer_metadata.y_start,
            layer_metadata.y_end,
        );
        let oscillator = Oscillator::new(Waveform::Square, frequencies[i], SAMPLE_RATE);
        section_interpreters.push(SectionInterpreter::new(oscillator, y_start, y_end));
        println!(
            "total_img_height: {}, y_start: {}, y_end: {}, frequency: {}",
            layer_metadata.total_img_height, y_start, y_end, frequencies[i]
        );
    }

    section_interpreters
}

fn clamp<T: Ord>(val: T, min: T, max: T) -> T {
    match val.cmp(&min) {
        Less => min,
        _ => match val.cmp(&max) {
            Greater => max,
            _ => val,
        },
    }
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
