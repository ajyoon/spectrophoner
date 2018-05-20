pub trait SampleGenerator {
    fn get_samples(&mut self, num: usize, amplitude: f32) -> Vec<f32>;
}
