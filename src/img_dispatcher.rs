use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

use image;
use image::{GenericImage, ImageBuffer, RgbImage, Rgb};
use ndarray::prelude::*;

/// Type alias for image channel identifiers
pub type ImgChannelId = u16;

/// A multiplexed image data packet containing multiple 8-bit image channels
///
/// This is a type-alias for a hashmap from channel ids to image data
/// represented as a 2D array of u8's.
pub type ImgPacket = HashMap<ImgChannelId, Array2<u8>>;

#[derive(Debug, Copy, Clone)]
pub struct ImgChannelMetadata {
    pub img_channel_id: ImgChannelId,
    // Coordinates are relative to the complete image's space,
    // meaning care must be taken to ensure these values are
    // within the bounds of the image section.
    pub y_start: usize,
    pub y_end: usize,
    // Size of the overall image
    pub total_img_height: usize,
}

struct StaticImgDispatcher {
    pub interpreter_data: Vec<(Vec<ImgChannelMetadata>, Receiver<ImgPacket>)>,
    sender_data: Vec<(Vec<ImgChannelMetadata>, Sender<ImgPacket>)>,
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    chunk_width: usize,
}


impl StaticImgDispatcher {

    pub fn new(path: &Path, chunk_width: usize) -> StaticImgDispatcher {
        let img = image::open(path).unwrap()
            .to_rgb();

        // begin wip hack...
        // for now just hard-code a single channel - later this will be
        // extracted into a private method which does the cool stuff
        let (packet_sender, packet_receiver) = channel::<ImgPacket>();

        let metadata = vec![ImgChannelMetadata {
            img_channel_id: 0,
            y_start: 0,
            y_end: img.height() as usize,
            total_img_height: img.height() as usize,
        }];

        let interpreter_data = vec![(metadata.clone(), packet_receiver)];
        let sender_data = vec![(metadata, packet_sender)];
        // ...end wip hack

        StaticImgDispatcher {
            interpreter_data,
            sender_data,
            img,
            chunk_width,
        }
    }

    fn dispatch_image(&self) {
    }
}
