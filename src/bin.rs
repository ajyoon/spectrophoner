extern crate clap;
extern crate num;
extern crate spectrophoner;

use std::time::Duration;
use std::thread;
use std::path::Path;

use clap::{Arg, App, SubCommand, ArgMatches};

use spectrophoner::conductor;
use spectrophoner::audio_streamer::AudioStreamer;
use spectrophoner::portaudio_streamer::PortAudioStreamer;
use spectrophoner::wav_streamer::WavStreamer;

const OUT_PATH_ARG: &str = "OUT_PATH";

pub fn main() {
    let matches = App::new("spectrophoner")
        .arg(Arg::with_name(OUT_PATH_ARG)
             .short("o")
             .long("output")
             .help("Target path to write audio, must be *.wav")
             .required(false)
             .takes_value(true))
        .get_matches();

    match matches.value_of(OUT_PATH_ARG) {
        Some(path) => {
            conductor::conduct(WavStreamer::<f32>::new(path.to_string()));
        },
        None => {
            conductor::conduct(PortAudioStreamer::new());
            thread::sleep(Duration::from_millis(1000_000));
        }
    }
}
