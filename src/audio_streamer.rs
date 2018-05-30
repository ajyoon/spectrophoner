use std::sync::mpsc::Receiver;

pub trait AudioStreamer<T> {
    fn stream(&self, chunk_receiver: Receiver<Vec<T>>);
}
