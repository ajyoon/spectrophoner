#![feature(custom_attribute)]
#![feature(test)]

extern crate rand;
extern crate image;
#[macro_use(array)]
#[macro_use(s)]
extern crate ndarray;
extern crate portaudio;
extern crate stopwatch;

pub mod arrays;
pub mod audio;
pub mod img_dispatcher;
pub mod img_interpreter;
pub mod mixer;
pub mod sample_buffer;
pub mod synth;
pub mod conductor;
mod pitch;

#[cfg(test)]
mod test_utils;
