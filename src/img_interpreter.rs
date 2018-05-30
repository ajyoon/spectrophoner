#[macro_use(array)]
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use ndarray::prelude::*;
use stopwatch::Stopwatch;

use img_dispatcher::{ImgLayerId, ImgLayerMetadata, ImgPacket};
use mixer;
use synth::Oscillator;

pub struct SectionInterpreter {
    pub oscillator: Oscillator,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    pub y_start: usize,
    pub y_end: usize,
    last_amplitude: f32,
}

pub struct ImgInterpreter {
    img_packet_receiver: Receiver<ImgPacket>,
    samples_sender: Sender<Vec<f32>>,
    samples_per_pixel: usize,
    layer_handlers: HashMap<ImgLayerId, Vec<SectionInterpreter>>,
}

#[inline]
fn amplitude_from_img_data(img_data: &ArrayView2<u8>) -> f32 {
    let mut sum = 0.0;
    for val in img_data {
        sum += *val as f32;
    }
    (sum / img_data.len() as f32) / (u8::max_value() as f32)
}

impl SectionInterpreter {
    pub fn new(oscillator: Oscillator, y_start: usize, y_end: usize) -> SectionInterpreter {
        SectionInterpreter {
            oscillator,
            y_start,
            y_end,
            last_amplitude: 0.,
        }
    }

    fn horizontally_slice_img_data<'a>(
        img_data: &'a Array2<u8>,
        y_start: usize,
        y_end: usize,
    ) -> ArrayView2<u8> {
        img_data.slice(s![.., y_start..y_end])
    }

    fn interpret(&mut self, num_samples: usize, img_data: &Array2<u8>) -> Vec<f32> {
        let slice = Self::horizontally_slice_img_data(img_data, self.y_start, self.y_end);
        let end_amplitude = amplitude_from_img_data(&slice);

        let samples = self.oscillator.get_samples_with_interpolated_amp(
            num_samples,
            self.last_amplitude,
            end_amplitude,
        );
        self.last_amplitude = end_amplitude;
        samples
    }
}

impl ImgInterpreter {
    pub fn new(
        img_packet_receiver: Receiver<ImgPacket>,
        samples_sender: Sender<Vec<f32>>,
        samples_per_pixel: usize,
        layer_handlers: HashMap<ImgLayerId, Vec<SectionInterpreter>>,
    ) -> ImgInterpreter {
        ImgInterpreter {
        img_packet_receiver,
        samples_sender,
        samples_per_pixel,
        layer_handlers,
        }
    }

    /// Start interpreting image data from img_packet_receiver into samples
    /// This loops forever until the img_packet_receiver is closed.
    pub fn interpret(&mut self) {
        for img_packet in &self.img_packet_receiver {
            let samples_needed =
                img_packet.values().nth(0).unwrap().len_of(Axis(0)) * self.samples_per_pixel;
            let mut mixed_samples = vec![0.; samples_needed];
            for (layer_id, img_data) in img_packet {
                let mut interpreters_for_layer = self.layer_handlers.get_mut(&layer_id).unwrap();
                for section_interpreter in interpreters_for_layer.iter_mut() {
                    let section_samples = section_interpreter.interpret(samples_needed, &img_data);
                    mixer::add_chunk_to(&section_samples, &mut mixed_samples);
                }
            }
            &self.samples_sender.send(mixed_samples);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn amplitude_from_img_data_all_zeros() {
        let img_data = array![[0, 0], [0, 0]];
        assert_almost_eq(amplitude_from_img_data(&img_data.view()), 0.);
    }

    #[test]
    fn amplitude_from_img_data_all_max() {
        let img_data = array![[255, 255], [255, 255]];
        assert_almost_eq(amplitude_from_img_data(&img_data.view()), 1.);
    }

    #[test]
    fn amplitude_from_img_data_avg_point_5() {
        let img_data = array![[0, 127], [128, 255]];
        assert_almost_eq(amplitude_from_img_data(&img_data.view()), 0.5);
    }

    #[test]
    fn test_horizontally_slice_img_data() {
        let img_data = array![
            [0, 0, 0],
            [1, 1, 1],
            [2, 2, 2],
        ];
        let slice = SectionInterpreter::horizontally_slice_img_data(&img_data, 0, 2);
        let expected = array![
            [0, 0],
            [1, 1],
            [2, 2],
        ];
        assert_img_data_eq_by_element(slice, expected.view());
    }
}

#[cfg(test)]
mod benchmarks {
    extern crate rand;
    extern crate test;
    use super::*;

    #[bench]
    fn bench_amplitude_from_image_data(b: &mut test::Bencher) {
        let image_data = random_image_data(1_000, 1_000);
        let image_view = image_data.view();
        b.iter(|| {
            amplitude_from_img_data(&image_view);
        });
    }

    fn random_image_data(width: usize, height: usize) -> Array2<u8> {
        let mut random_array = Array2::zeros((width, height));
        for element in random_array.iter_mut() {
            *element = rand::random::<u8>();
        }
        random_array
    }
}
