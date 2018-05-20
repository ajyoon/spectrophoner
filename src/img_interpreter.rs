use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use ndarray::prelude::*;

use synth::Oscillator;

/// Type alias for image channel identifiers
type ImgChannelId = u16;

/// Type alias for a map between image channel ids
/// and 2d arrays of greyscale pixels.
type ImgChannels = HashMap<ImgChannelId, Array2<u8>>;

/// A function responsible for generating oscillators
/// when given information about where they exist in
/// an image slice.
///
/// # Arguments
///
/// * `y_pos_ratio`: A ratio (between 0 and 1) which indicates how far
///   from the top of an image an oscillator occupies.
/// * `height_ratio`: A ratio (between 0 and 1) which indicates how much
///   of the image is covered by the oscillator.
///
/// # Remarks
///
/// `y_pos_ratio + height_ratio <= 1.0` should always hold.
///
/// The ratios are within the full image's space
type OscillatorGenerator = fn(
    y_pos_ratio: f32,
    height_ratio: f32,
) -> Oscillator;


pub struct SectionInterpreter {
    oscillator: Oscillator,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    y_start: usize,
    y_end: usize,
}

pub struct ImgInterpreter {
    img_channels_receiver: Receiver<ImgChannels>,
    samples_sender: Sender<Vec<f32>>,
    pixels_per_second: f32,
    channel_handlers: HashMap<ImgChannelId, SectionInterpreter>,
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

impl ImgInterpreter {
    pub fn new(
        img_channels_metadata: Vec<ImgChannelMetadata>,
        img_channels_receiver: Receiver<ImgChannels>,
        samples_sender: Sender<Vec<f32>>,
        pixels_per_second: f32,
        oscillator_generators: HashMap<ImgChannelId, OscillatorGenerator>
    ) -> ImgInterpreter {

        let mut channel_handlers = HashMap::<ImgChannelId, SectionInterpreter>::new();

        for metadata in img_channels_metadata {
            let y_pos_ratio = metadata.y_start as f32 / metadata.total_img_height as f32;
            let height_ratio = (metadata.y_end - metadata.y_start) as f32
                / metadata.total_img_height as f32;
            let oscillator = oscillator_generators.get(&metadata.img_channel_id).unwrap()(
                y_pos_ratio, height_ratio
            );
            let section_interpreter = SectionInterpreter {
                oscillator,
                y_start: metadata.y_start,
                y_end: metadata.y_end
            };
            channel_handlers.insert(metadata.img_channel_id, section_interpreter);
        }

        ImgInterpreter {
            img_channels_receiver,
            samples_sender,
            pixels_per_second,
            channel_handlers
        }
    }
}
