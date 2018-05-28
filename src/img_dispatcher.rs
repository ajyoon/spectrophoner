use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

use image;
use image::imageops::colorops;
use image::{GenericImage, ImageBuffer, Rgb, RgbImage, SubImage};
use ndarray::prelude::*;

/// Type alias for image channel identifiers
pub type ImgLayerId = u16;

/// A multiplexed image data packet containing multiple 8-bit image channels
///
/// This is a type-alias for a hashmap from channel ids to image data
/// represented as a 2D array of u8's.
pub type ImgPacket = HashMap<ImgLayerId, Array2<u8>>;

pub type RgbImage24Bit = ImageBuffer<Rgb<u8>, Vec<u8>>;
pub type RgbImage24BitSlice<'a> = SubImage<'a, ImageBuffer<Rgb<u8>, Vec<u8>>>;

#[derive(Debug, Copy, Clone)]
pub struct ImgLayerMetadata {
    pub img_layer_id: ImgLayerId,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    pub y_start: usize,
    pub y_end: usize,
    // Size of the overall image
    pub total_img_height: usize,
}

type LayerExtractorFn = fn(&RgbImage24BitSlice) -> Array2<u8>;

pub struct ChannelExporter {
    pub receiver: Receiver<ImgPacket>,
    pub layers_metadata: Vec<ImgLayerMetadata>,
}

/// Responsible for extracting data from the primary image dispatcher input,
/// separating into layers, and sending chunks through a channel
/// where it may be picked up by image interpreters
struct ChannelHandler {
    sender: Sender<ImgPacket>,
    layer_extractors: HashMap<ImgLayerId, LayerExtractorFn>,
}

impl ChannelHandler {
    fn dispatch_channel(&self, img: &RgbImage24BitSlice) {
        let mut packet = ImgPacket::new();
        for (layer_id, layer_extractor_fn) in &self.layer_extractors {
            packet.insert(*layer_id, layer_extractor_fn(img));
        }
        self.sender.send(packet).unwrap();
    }
}

/// Responsible for managing a series of channels via ChannelHandlers
pub struct StaticImgDispatcher {
    channel_handlers: Vec<ChannelHandler>,
    img: RgbImage24Bit,
    chunk_width: u32,
}

impl StaticImgDispatcher {
    pub fn new(path: &Path, chunk_width: u32) -> (StaticImgDispatcher, Vec<ChannelExporter>) {
        let img = image::open(path).unwrap().to_rgb();

        let mut channel_handlers = Vec::<ChannelHandler>::new();
        let mut channel_exporters = Vec::<ChannelExporter>::new();

        for (handler, exporter) in Self::generate_channels(&img) {
            channel_handlers.push(handler);
            channel_exporters.push(exporter);
        }

        (
            StaticImgDispatcher {
                channel_handlers,
                img,
                chunk_width,
            },
            channel_exporters,
        )
    }

    fn generate_channels(img: &RgbImage24Bit) -> Vec<(ChannelHandler, ChannelExporter)> {
        // naive initial implementation using just 1 channel with just 1 layer
        let (sender, receiver) = channel::<ImgPacket>();
        let layers_metadata = vec![ImgLayerMetadata {
            img_layer_id: 0,
            y_start: 0,
            y_end: img.height() as usize,
            total_img_height: img.height() as usize,
        }];
        let mut layer_extractors = HashMap::<ImgLayerId, LayerExtractorFn>::new();
        layer_extractors.insert(0, naive_layer_extractor);

        vec![(
            ChannelHandler {
                sender,
                layer_extractors,
            },
            ChannelExporter {
                receiver,
                layers_metadata,
            },
        )]
    }

    /// Send chunks of image data through channels until the image is fully consumed.
    pub fn begin_dispatch(&mut self) {
        let mut current_x = 0;
        let chunk_width = self.chunk_width;
        let img_width = self.img.width();
        loop {
            if current_x + self.chunk_width >= img_width {
                self.dispatch_slice(current_x, img_width - current_x);
                break;
            } else {
                self.dispatch_slice(current_x, chunk_width);
                current_x += chunk_width;
            }
        }
    }

    fn dispatch_slice(&mut self, start_x: u32, width: u32) {
        let img_height = self.img.height();
        let slice = self.img.sub_image(start_x, 0, width, img_height);
        for channel_handler in &self.channel_handlers {
            channel_handler.dispatch_channel(&slice);
        }
    }
}

pub fn naive_layer_extractor(img: &RgbImage24BitSlice) -> Array2<u8> {
    let grayscale = colorops::grayscale(img);
    Array::from_shape_vec(
        (img.width() as usize, img.height() as usize).strides((1, img.width() as usize)),
        grayscale.into_raw(),
    ).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image;
    use test_utils::*;

    #[test]
    fn test_naive_layer_extractor() {
        // also test some assumptions about image -> ndarray mappings

        let width = 3;
        let height = 2;

        let mut buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);
        buffer.put_pixel(0, 0, image::Rgb([0, 0, 0]));
        buffer.put_pixel(1, 0, image::Rgb([1, 1, 1]));
        buffer.put_pixel(2, 0, image::Rgb([2, 2, 2]));
        buffer.put_pixel(0, 1, image::Rgb([3, 3, 3]));
        buffer.put_pixel(1, 1, image::Rgb([4, 4, 4]));
        buffer.put_pixel(2, 1, image::Rgb([5, 5, 5]));

        let full_size_slice = buffer.sub_image(0, 0, width, height);

        let extracted_layer = naive_layer_extractor(&full_size_slice);

        #[rustfmt_skip]
        let expected_layer_array = array![
            [0, 3],
            [1, 4],
            [2, 5]
        ];
        assert_img_data_eq_by_element(extracted_layer.view(), expected_layer_array.view());

        assert_eq!(extracted_layer.len_of(Axis(0)), width as usize);
        assert_eq!(extracted_layer.len_of(Axis(1)), height as usize);

        assert_eq!(extracted_layer.get((0, 0)).unwrap(), &0u8);
        assert_eq!(extracted_layer.get((1, 0)).unwrap(), &1u8);
        assert_eq!(extracted_layer.get((2, 0)).unwrap(), &2u8);
        assert_eq!(extracted_layer.get((0, 1)).unwrap(), &3u8);
        assert_eq!(extracted_layer.get((1, 1)).unwrap(), &4u8);
        assert_eq!(extracted_layer.get((2, 1)).unwrap(), &5u8);
    }
}
