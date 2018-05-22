extern crate rand;
extern crate image;
#[macro_use(array)]
extern crate ndarray;
extern crate portaudio;

mod util;
mod synth;
mod arrays;
mod chunk;
mod mixer;
mod img_dispatcher;
mod sample_generator;
mod img_interpreter;
mod audio;
mod conductor;
