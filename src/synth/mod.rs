use std::f64::consts;

use util::PosF32;

const TWO_PI: f64 = consts::PI * 2.;

fn period_length(frequency: PosF32, sample_rate: u32) -> u32 {
    return sample_rate / frequency;
}

trait PeriodGenerator {
    fn generate_period(&self, frequency: PosF32, sample_rate: u32) -> Vec<i16>;
}

enum Waveform {
    Sine,
    Square,
}

#[inline]
fn populate_sine_period(period: &mut Vec<i16>) {
    let x_scale = TWO_PI / (period.capacity() as f64);
    let y_scale = i16::max_value() as f64;
    for i in 0..period.capacity() {
        period.push((((i as f64) * x_scale).sin() * (y_scale)) as i16);
    }
}

#[inline]
fn populate_square_period(period: &mut Vec<i16>) {
    let high_len = period.capacity() / 2;
    let low_len = period.capacity() - high_len;
    let high_val = i16::max_value();
    let low_val = i16::min_value();
    for _ in 0..high_len {
        period.push(high_val);
    }
    for _ in 0..low_len {
        period.push(low_val);
    }
}

impl PeriodGenerator for Waveform {
    fn generate_period(&self, frequency: PosF32, sample_rate: u32) -> Vec<i16> {
        let samples_needed = period_length(frequency, sample_rate);
        let mut period = Vec::<i16>::with_capacity(samples_needed as usize);

        match self {
            &Waveform::Sine => populate_sine_period(&mut period),
            &Waveform::Square => populate_square_period(&mut period),
        };
        return period;
    }
}

trait SampleGenerator {
    fn get_samples(&mut self, num: u32, amplitude: f64) -> Vec<i16>;
}

struct Oscillator {
    period_cache: Vec<i16>,
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
    fn get_samples(&mut self, num: u32, amplitude: f64) -> Vec<i16> {
        let mut samples = Vec::<i16>::with_capacity(num as usize);
        let period_len = &self.period_cache.len();
        for i in 0..samples.capacity() {
            samples.push(
                ((self.period_cache[(&self.phase + i) % period_len] as f64) * amplitude) as i16,
            )
        }
        self.phase = (&self.phase + samples.capacity()) % period_len;
        return samples;
    }
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;

    mod waveforms {
        use super::*;

        mod sine {
            use super::*;

            #[test]
            fn compare_against_known_good_output() {
                let actual = Waveform::Sine.generate_period(PosF32::new(2250.), 44100);
                let expected = vec![
                    // these values verified as sane by plotting and doing an eyeball check
                    0, 10639, 20125, 27431, 31764, 32655, 30007, 24107, 15595, 5393, -5393,
                    -15595, -24107, -30007, -32655, -31764, -27431, -20125, -10639,
                ];
                assert_eq!(actual, expected);
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
                    32767, 32767, 32767, 32767, 32767, -32768, -32768, -32768, -32768, -32768];
                assert_eq!(actual, expected);
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
        fn test() {
            let mut osc = Oscillator::new(Waveform::Sine, PosF32::new(440.), 44100);
            let start = time::precise_time_ns();
            let sample_count = 1_000_000;
            let samples = osc.get_samples(sample_count, 1.);
            println!("{}", samples[0]);
            let elapsed_ms = (time::precise_time_ns() - start) / 1_000_000;

            println!(
                "generated {} samples ({}s worth) in {} ms",
                sample_count,
                sample_count / 44100,
                elapsed_ms
            );
            assert!(false);
        }
    }
}
