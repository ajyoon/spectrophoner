extern crate libc;

use std::f32::consts;

use util::PosF32;
use arrays;

const TWO_PI: f32 = consts::PI * 2.;
const SINGLE_SIGNAL_MIN: f32 = -1.;
const SINGLE_SIGNAL_MAX: f32 = 1.;

fn period_length(frequency: PosF32, sample_rate: u32) -> u32 {
    return sample_rate / frequency;
}

trait PeriodGenerator {
    fn generate_period(&self, frequency: PosF32, sample_rate: u32) -> Vec<f32>;
}

enum Waveform {
    Sine,
    Square,
}

#[inline]
fn populate_sine_period(period: &mut Vec<f32>) {
    let x_scale = TWO_PI / (period.capacity() as f32);
    let y_scale = SINGLE_SIGNAL_MAX;
    for i in 0..period.capacity() {
        period.push(((i as f32) * x_scale).sin() * (y_scale));
    }
}

#[inline]
fn populate_square_period(period: &mut Vec<f32>) {
    let high_len = period.capacity() / 2;
    let low_len = period.capacity() - high_len;
    for _ in 0..high_len {
        period.push(SINGLE_SIGNAL_MAX);
    }
    for _ in 0..low_len {
        period.push(SINGLE_SIGNAL_MIN);
    }
}

impl PeriodGenerator for Waveform {
    fn generate_period(&self, frequency: PosF32, sample_rate: u32) -> Vec<f32> {
        let samples_needed = period_length(frequency, sample_rate);
        let mut period = Vec::<f32>::with_capacity(samples_needed as usize);

        match self {
            &Waveform::Sine => populate_sine_period(&mut period),
            &Waveform::Square => populate_square_period(&mut period),
        };
        return period;
    }
}

trait SampleGenerator {
    fn get_samples(&mut self, num: u32, amplitude: f32) -> Vec<f32>;
}

struct Oscillator {
    period_cache: Vec<f32>,
    phase: usize,
}

impl Oscillator {
    fn new(waveform: Waveform, frequency: PosF32, sample_rate: u32) -> Oscillator {
        Oscillator {
            period_cache: waveform.generate_period(frequency, sample_rate),
            phase: 0,
        }
    }
}

impl SampleGenerator for Oscillator {
    fn get_samples(&mut self, num: u32, amplitude: f32) -> Vec<f32> {
        let period_at_amplitude = &self.period_cache.iter().map(|s| (*s * amplitude)).collect();
        let samples = arrays::roll_vec(&period_at_amplitude, self.phase, num as usize);
        self.phase = (&self.phase + num as usize) % &self.period_cache.len();
        samples
    }
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;

    fn assert_array_almost_eq_by_element(left: Vec<f32>, right: Vec<f32>) {
        const F32_EPSILON: f32 = 1.0e-6;
        if left.len() != right.len() {
            panic!(
                "lengths differ: left.len() = {}, right.len() = {}",
                left.len(),
                right.len()
            );
        }
        for (left_val, right_val) in left.iter().zip(right.iter()) {
            assert!(
                (*left_val - *right_val).abs() < F32_EPSILON,
                "{} is not approximately equal to {}. \
                 complete left vec: {:?}. complete right vec: {:?}",
                *left_val,
                *right_val,
                left,
                right
            );
        }
    }

    mod waveforms {
        use super::*;

        mod sine {
            use super::*;

            #[test]
            fn compare_against_known_good_output() {
                let actual = Waveform::Sine.generate_period(PosF32::new(2250.), 44100);
                let expected = vec![
                    // these values verified as sane by plotting and doing an eyeball check
                    0.0, 0.32469946, 0.6142127, 0.8371665, 0.9694003, 0.9965845, 0.91577333,
                    0.7357239, 0.4759474, 0.16459462, -0.16459456, -0.47594735, -0.7357239,
                    -0.9157734, -0.9965845, -0.96940035, -0.8371665, -0.6142126, -0.32469952
                ];
                assert_array_almost_eq_by_element(actual, expected);
            }

            #[test]
            fn capacity_used() {
                let period = Waveform::Sine.generate_period(PosF32::new(10.), 44100);
                assert_eq!(period.len(), period.capacity());
            }
        }

        mod square {
            use super::*;

            #[test]
            fn compare_against_known_good_output() {
                let actual = Waveform::Square.generate_period(PosF32::new(4410.), 44100);
                let expected = vec![
                    // these values verified as sane by plotting and doing an eyeball check
                    1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0];
                assert_array_almost_eq_by_element(actual, expected);
            }

            #[test]
            fn capacity_used() {
                let period = Waveform::Square.generate_period(PosF32::new(10.), 44100);
                assert_eq!(period.len(), period.capacity());
            }
        }
    }

    mod oscillator {
        use super::*;

        #[test]
        #[ignore]
        fn test() {
            let mut osc = Oscillator::new(Waveform::Sine, PosF32::new(440.), 44100);
            let start = time::precise_time_ns();
            let sample_count = 10_000_000;
            let samples = osc.get_samples(sample_count, 1.);
            println!("{}", samples[0]);
            let elapsed_ms = (time::precise_time_ns() - start) / 1_000_000;

            println!(
                "generated {} samples ({}s worth) in {} ms",
                sample_count,
                sample_count / 44100,
                elapsed_ms
            );
            // 250_000_000 / s
            assert!(false);
        }

        #[test]
        fn t() {}
    }
}
