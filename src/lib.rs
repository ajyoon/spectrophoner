#![feature(vec_resize_default)]
#![feature(test)]

extern crate rand;
extern crate image;
#[macro_use(array)]
extern crate ndarray;
extern crate portaudio;

mod util;
mod synth;
mod arrays;
mod mixer;
mod img_dispatcher;
mod sample_generator;
mod img_interpreter;
mod audio;
pub mod conductor;

#[cfg(test)]
mod test_utils;
