use std::fs::File;
use std::path::Path;

use image;
use image::{GenericImage, ImageBuffer, RgbImage};

pub trait ImageLoader {
    fn load_image(&self) -> RgbImage;
}

pub struct StaticImageLoader {
    path: Path
}

impl StaticImageLoader {

}

impl ImageLoader for StaticImageLoader {
    fn load_image(&self) -> RgbImage {
        let img = image::open(&self.path).unwrap();
        img.to_rgb()
    }
}
