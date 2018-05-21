use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

use image;
use image::imageops::colorops;
use image::{GenericImage, ImageBuffer, RgbImage, Rgb, SubImage};
use ndarray::prelude::*;

/// Type alias for image channel identifiers
pub type ImgLayerId = u16;

/// A multiplexed image data packet containing multiple 8-bit image channels
///
/// This is a type-alias for a hashmap from channel ids to image data
/// represented as a 2D array of u8's.
pub type ImgPacket = HashMap<ImgLayerId, Array2<u8>>;

type RgbImage24Bit = ImageBuffer<Rgb<u8>, Vec<u8>>;
type RgbImage24BitSlice<'a> = SubImage<'a, ImageBuffer<Rgb<u8>, Vec<u8>>>;

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

/// Responsible for extracting data from the primary image dispatcher input,
/// separating into layers, and sending chunks through a channel
/// where it may be picked up by image interpreters
struct ChannelHandler {
    receiver: Receiver<ImgPacket>,
    sender: Sender<ImgPacket>,
    layer_extractors: HashMap<ImgLayerId, LayerExtractorFn>,
    layers_metadata: Vec<ImgLayerMetadata>
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
struct StaticImgDispatcher {
    // pub interpreter_data: Vec<(Vec<ImgLayerMetadata>, Receiver<ImgPacket>)>,
    // sender_data: Vec<(Vec<ImgLayerMetadata>, Sender<ImgPacket>)>,
    pub channel_handlers: Vec<ChannelHandler>,
    img: RgbImage24Bit,
    chunk_width: u32,
}


impl StaticImgDispatcher {

    pub fn new(path: &Path, chunk_width: u32) -> StaticImgDispatcher {
        let img = image::open(path).unwrap().to_rgb();

        let channel_handlers = Self::generate_channel_handlers(&img);

        StaticImgDispatcher {
            img,
            chunk_width,
            channel_handlers
        }
    }

    fn generate_channel_handlers(img: &RgbImage24Bit) -> Vec<ChannelHandler> {
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

        vec![ChannelHandler {
            receiver,
            sender,
            layer_extractors,
            layers_metadata
        }]
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


fn naive_layer_extractor(img: &RgbImage24BitSlice) -> Array2<u8> {
    let grayscale = colorops::grayscale(img);
    Array::from_shape_vec(
        (img.width() as usize, img.height() as usize),
        grayscale.into_raw()
    ).unwrap()
}