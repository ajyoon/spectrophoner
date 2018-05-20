#[macro_use(array)]
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use ndarray::prelude::*;

use mixer;
use sample_generator::SampleGenerator;
use synth::Oscillator;

/// Type alias for image channel identifiers
type ImgChannelId = u16;

/// A multiplexed image data packet containing multiple 8-bit image channels
///
/// This is a type-alias for a hashmap from channel ids to image data
/// represented as a 2D array of u8's.
type ImagePacket = HashMap<ImgChannelId, Array2<u8>>;

pub struct SectionInterpreter {
    oscillator: Oscillator,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    y_start: usize,
    y_end: usize,
}

type SectionInterpreterGenerator =
    fn(y_pos_ratio: f32, height_ratio: f32, total_img_height: usize) -> Vec<SectionInterpreter>;

pub struct ImgInterpreter {
    img_channels_receiver: Receiver<ImagePacket>,
    samples_sender: Sender<Vec<f32>>,
    samples_per_img_chunk: usize,
    channel_handlers: HashMap<ImgChannelId, Vec<SectionInterpreter>>,
}

pub struct ImgChannelMetadata {
    img_channel_id: ImgChannelId,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    y_start: usize,
    y_end: usize,
    // Size of the overall image
    total_img_height: usize,
}

fn amplitude_from_img_data(img_data: &Array2<u8>) -> f32 {
    let mut sum = 0.0;
    for val in img_data {
        sum += *val as f32;
    }
    (sum / img_data.len() as f32) / (u8::max_value() as f32)
}

impl SectionInterpreter {
    fn interpret(&mut self, num_samples: usize, img_data: &Array2<u8>) -> Vec<f32> {
        let amplitude = amplitude_from_img_data(img_data);
        self.oscillator.get_samples(num_samples, amplitude)
    }
}

impl ImgInterpreter {
    pub fn new(
        img_channels_metadata: Vec<ImgChannelMetadata>,
        img_channels_receiver: Receiver<ImagePacket>,
        samples_sender: Sender<Vec<f32>>,
        samples_per_img_chunk: usize,
        section_interpreter_generators: HashMap<ImgChannelId, SectionInterpreterGenerator>,
    ) -> ImgInterpreter {
        let mut channel_handlers = HashMap::<ImgChannelId, Vec<SectionInterpreter>>::new();

        for metadata in img_channels_metadata {
            let y_pos_ratio = metadata.y_start as f32 / metadata.total_img_height as f32;
            let height_ratio =
                (metadata.y_end - metadata.y_start) as f32 / metadata.total_img_height as f32;
            let section_interpreters =
                section_interpreter_generators
                    .get(&metadata.img_channel_id)
                    .unwrap()(y_pos_ratio, height_ratio, metadata.total_img_height);
            channel_handlers.insert(metadata.img_channel_id, section_interpreters);
        }

        ImgInterpreter {
            img_channels_receiver,
            samples_sender,
            samples_per_img_chunk,
            channel_handlers,
        }
    }

    /// Start interpreting image data from img_channels_receiver into samples
    /// This loops forever until the img_channels_receiver is closed.
    pub fn interpret(&mut self) {
        for channels in &self.img_channels_receiver {
            let mut mixed_samples = Vec::new();
            for (channel_id, img_data) in channels {
                let mut interpreters_for_channel =
                    self.channel_handlers.get_mut(&channel_id).unwrap();
                for section_interpreter in interpreters_for_channel.iter_mut() {
                    let section_samples =
                        section_interpreter.interpret(self.samples_per_img_chunk, &img_data);
                    mixer::add_samples(mixed_samples.as_mut_slice(), section_samples.as_slice());
                }
            }
            &self.samples_sender.send(mixed_samples);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_almost_eq(left: f32, right: f32) {
        const F32_EPSILON: f32 = 1.0e-6;
        assert!(
            (left - right).abs() < F32_EPSILON,
            "{} is not approximately equal to {}.",
            left,
            right,
        );
    }

    #[test]
    fn amplitude_from_img_data_all_zeros() {
        let img_data = array![[0, 0], [0, 0]];
        assert_almost_eq(amplitude_from_img_data(&img_data), 0.);
    }

    #[test]
    fn amplitude_from_img_data_all_max() {
        let img_data = array![[255, 255], [255, 255]];
        assert_almost_eq(amplitude_from_img_data(&img_data), 1.);
    }

    #[test]
    fn amplitude_from_img_data_avg_point_5() {
        let img_data = array![[0, 127], [128, 255]];
        assert_almost_eq(amplitude_from_img_data(&img_data), 0.5);
    }
}
