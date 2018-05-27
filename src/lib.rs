#![feature(vec_resize_default)]
#![feature(test)]

extern crate rand;
extern crate image;
#[macro_use(array)]
extern crate ndarray;
extern crate portaudio;
extern crate stopwatch;

mod arrays;
mod audio;
mod img_dispatcher;
mod img_interpreter;
mod mixer;
mod sample_buffer;
mod sample_generator;
mod synth;
mod util;

pub mod conductor;

#[cfg(test)]
mod test_utils;
