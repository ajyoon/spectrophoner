#![feature(custom_attribute)]
#![feature(test)]

extern crate rand;
extern crate image;
#[macro_use(array)]
#[macro_use(s)]
extern crate ndarray;
extern crate portaudio;
extern crate stopwatch;
extern crate hound;

mod arrays;
mod img_dispatcher;
mod img_interpreter;
mod mixer;
mod sample_buffer;
mod synth;
mod pitch;
mod audio_streamer;
mod portaudio_streamer;
mod wav_streamer;

pub mod conductor;

#[cfg(test)]
mod test_utils;
