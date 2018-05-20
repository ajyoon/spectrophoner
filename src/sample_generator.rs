pub trait SampleGenerator {
    fn get_samples(&mut self, num: u32, amplitude: f32) -> Vec<f32>;
}
