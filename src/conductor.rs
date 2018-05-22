use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

use img_dispatcher::{ChannelHandler, StaticImgDispatcher};
use img_interpreter::{ImgInterpreter, SectionInterpreterGenerator};

/// hacky testing for now
pub fn conduct() {
    let chunk_width = 10;
    let img_path = Path::new("../resources/ascending_line.png");
    let img_dispatcher = StaticImgDispatcher::new(img_path, chunk_width);

    thread::spawn(move || {
        img_dispatcher.begin_dispatch();
    });

    for channel_handler in img_dispatcher.channel_handlers {
        let (samples_sender, samples_receiver) = channel::<Vec<f32>>();
        let interpreter = interpreter_for_channel_handler(channel_handler, samples_sender);
    }

    ()
}

fn interpreter_for_channel_handler(
    channel_handler: ChannelHandler,
    samples_sender: Sender<Vec<f32>>,
) -> ImgInterpreter {
    const TEMP_HARDCODED_SAMPLES_PER_IMG_CHUNK: usize = 441000;

    let section_interpreter_generators =
        derive_simple_section_interpreter_generators(channel_handler);

    ImgInterpreter::new(
        channel_handler.layers_metadata,
        channel_handler.receiver,
        samples_sender,
        TEMP_HARDCODED_SAMPLES_PER_IMG_CHUNK,
        section_interpreter_generators,
    )
}

fn naive_section_interpreter_generator(
    y_pos_ratio: f32,
    height_ratio: f32,
    total_img_height: usize,
) -> Vec<SectionInterpreter> {

}

fn derive_simple_section_interpreter_generators(
    channel_handler: ChannelHandler,
) -> HashMap<ImgLayerId, SectionInterpreterGenerator> {
    let generators = HashMap::new();
    for metadata in channel_handler.layers_metadata {
        generators.insert(metadata.img_layer_id, naive_section_interpreter_generator);
    }
    generators
}
