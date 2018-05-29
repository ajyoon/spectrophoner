use std::f32::consts;

use ndarray::prelude::*;

use arrays;

const TWO_PI: f32 = consts::PI * 2.;
const SINGLE_SIGNAL_MIN: f32 = -1.;
const SINGLE_SIGNAL_MAX: f32 = 1.;

fn period_length(frequency: f32, sample_rate: u32) -> u32 {
    return ((sample_rate as f32) / frequency) as u32;
}

trait PeriodGenerator {
    fn generate_period(&self, frequency: f32, sample_rate: u32) -> Vec<f32>;
}

pub enum Waveform {
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
    fn generate_period(&self, frequency: f32, sample_rate: u32) -> Vec<f32> {
        assert!(frequency > 0., "Invalid frequency: {}", frequency);
        let samples_needed = period_length(frequency, sample_rate);
        let mut period = Vec::<f32>::with_capacity(samples_needed as usize);

        match self {
            &Waveform::Sine => populate_sine_period(&mut period),
            &Waveform::Square => populate_square_period(&mut period),
        };
        return period;
    }
}

pub struct Oscillator {
    period_cache: Vec<f32>,
    phase: usize,
}

impl Oscillator {
    pub fn new(waveform: Waveform, frequency: f32, sample_rate: u32) -> Oscillator {
        Oscillator {
            period_cache: waveform.generate_period(frequency, sample_rate),
            phase: 0,
        }
    }

    pub fn get_samples(&mut self, num: usize, amplitude: f32) -> Vec<f32> {
        let period_at_amplitude = &self.period_cache.iter().map(|s| (*s * amplitude)).collect();
        let samples = arrays::roll_vec(&period_at_amplitude, self.phase, num);
        self.phase = (&self.phase + num) % &self.period_cache.len();
        samples
    }

    pub fn get_samples_with_interpolated_amp(
        &mut self,
        num: usize,
        start_amplitude: f32,
        end_amplitude: f32,
    ) -> Vec<f32> {
        let mut samples = arrays::roll_vec(&self.period_cache, self.phase, num);
        arrays::multiply_over_linspace(samples.as_mut_slice(), start_amplitude, end_amplitude);
        self.phase = (&self.phase + num) % &self.period_cache.len();
        samples
    }
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;
    use test_utils::*;

    mod waveforms {
        use super::*;

        mod sine {
            use super::*;

            #[test]
            fn compare_against_known_good_output() {
                let actual = Waveform::Sine.generate_period(2250., 44100);
                #[rustfmt_skip]
                let expected = vec![
                    // these values verified as sane by plotting and doing an eyeball check
                    0.0, 0.32469946, 0.6142127, 0.8371665, 0.9694003, 0.9965845,
                    0.91577333, 0.7357239, 0.4759474, 0.16459462, -0.16459456,
                    -0.47594735, -0.7357239, -0.9157734, -0.9965845,
                    -0.96940035, -0.8371665, -0.6142126, -0.32469952,
                ];
                assert_almost_eq_by_element(actual, expected);
            }

            #[test]
            fn capacity_used() {
                let period = Waveform::Sine.generate_period(10., 44100);
                assert_eq!(period.len(), period.capacity());
            }
        }

        mod square {
            use super::*;

            #[test]
            fn compare_against_known_good_output() {
                let actual = Waveform::Square.generate_period(4410., 44100);
                #[rustfmt_skip]
                let expected = vec![
                    // these values verified as sane by plotting and doing an eyeball check
                    1.0, 1.0, 1.0, 1.0, 1.0,
                    -1.0, -1.0, -1.0, -1.0, -1.0,
                ];
                assert_almost_eq_by_element(actual, expected);
            }

            #[test]
            fn capacity_used() {
                let period = Waveform::Square.generate_period(10., 44100);
                assert_eq!(period.len(), period.capacity());
            }
        }
    }

    mod oscillator {
        use super::*;

        #[test]
        fn get_samples_preserves_phase() {
            let mut osc = Oscillator::new(Waveform::Sine, 4410., 44100);
            let mut samples = osc.get_samples(10, 1.);
            samples.append(&mut osc.get_samples(10, 1.));
            #[rustfmt_skip]
            let expected: Vec<f32> = vec![
                0.0, 0.58778524, 0.95105654, 0.9510565, 0.5877852,
                -0.00000008742278, -0.58778536, -0.9510565, -0.9510565, -0.58778495,
                0.0, 0.58778524, 0.95105654, 0.9510565, 0.5877852,
                -0.00000008742278, -0.58778536, -0.9510565, -0.9510565, -0.58778495,
            ];
            assert_almost_eq_by_element(samples, expected);
        }

        #[test]
        fn get_samples_with_interpolated_amp() {
            let mut osc = Oscillator::new(Waveform::Sine, 4410., 44100);
            let mut samples = osc.get_samples_with_interpolated_amp(20, 0., 1.);
            #[rustfmt_skip]
            let expected: Vec<f32> = vec![
                0.0, 0.030936066, 0.10011122, 0.15016681, 0.12374425,
                -0.000000023005995, -0.18561642, -0.3503892, -0.40044484,
                -0.27842444, 0.0, 0.34029672, 0.60066724, 0.65072286,
                0.43310487, -0.000000069017986, -0.49497715, -0.85094523,
                -0.9010009, -0.58778495
            ];
            assert_almost_eq_by_element(samples, expected);
        }

        #[test]
        #[ignore]
        fn test() {
            let mut osc = Oscillator::new(Waveform::Sine, 440., 44100);
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
