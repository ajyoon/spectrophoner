use std::cmp::Ordering;

pub struct Chunk {
    pub start_sample: usize,
    pub sender_id: u16,
    pub samples: Vec<f32>
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Chunk) -> bool {
        return self.start_sample == other.start_sample;
    }
}

impl Eq for Chunk {}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Chunk) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implement ordering on chunks based on start_sample
// so that, when placed in a priority queue, samples
// with the oldest start_sample are picked up first
impl Ord for Chunk {
    fn cmp(&self, other: &Chunk) -> Ordering {
        other.start_sample.cmp(&self.start_sample)
    }
}
