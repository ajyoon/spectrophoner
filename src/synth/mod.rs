extern crate libc;

use std::f64::consts;
use std::mem;

use util::PosF32;

const TWO_PI: f64 = consts::PI * 2.;

fn roll_vec(src_vec: &Vec<i16>, src_offset: usize, elements: usize) -> Vec<i16> {
    let element_size = mem::size_of::<i16>();
    let roll_offset = src_offset % src_vec.len();
    let mut rolled = Vec::<i16>::with_capacity(elements);

    let mut src_ptr;
    let mut write_ptr = rolled.as_slice().as_ptr() as usize;

    let mut copied_elements = 0;

    if roll_offset != 0 {
        src_ptr = src_vec.as_slice().split_at(roll_offset).1.as_ptr() as usize;
        let bytes = (src_vec.len() - roll_offset) * element_size;
        unsafe {
            libc::memcpy(
                write_ptr as *mut libc::c_void,
                src_ptr as *mut libc::c_void,
                bytes,
            );
        }
        write_ptr = write_ptr + bytes;
        copied_elements = src_vec.len() - roll_offset;
    }

    let full_rolls = (elements - copied_elements) / src_vec.len();
    let bytes_per_copy = src_vec.len() * element_size;
    src_ptr = src_vec.as_slice().as_ptr() as usize;
    for _ in 0..full_rolls {
        unsafe {
            libc::memcpy(
                write_ptr as *mut libc::c_void,
                src_ptr as *mut libc::c_void,
                bytes_per_copy,
            );
        }
        write_ptr = write_ptr + bytes_per_copy;
    }
    copied_elements += full_rolls * src_vec.len();

    let remaining_elements = elements - copied_elements;
    if remaining_elements > 0 {
        src_ptr = src_vec.as_slice().split_at(remaining_elements).0.as_ptr() as usize;
        let bytes = remaining_elements * element_size;
        unsafe {
            libc::memcpy(
                write_ptr as *mut libc::c_void,
                src_ptr as *mut libc::c_void,
                bytes,
            );
        }
    }

    unsafe {
        rolled.set_len(elements);
    }

    rolled
}

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
        let period_at_amplitude = &self.period_cache
            .iter()
            .map(|s| (*s as f64 * amplitude) as i16)
            .collect();
        let samples = roll_vec(&period_at_amplitude, self.phase, num as usize);
        self.phase = (&self.phase + num as usize) % &self.period_cache.len();
        samples
        // let mut samples = Vec::<i16>::with_capacity(num as usize);
        // let period_len = &self.period_cache.len();
        // for i in 0..samples.capacity() {
        //     samples.push(
        //         ((self.period_cache[(&self.phase + i) % period_len] as f64) * amplitude) as i16,
        //     )
        // }
        // self.phase = (&self.phase + samples.capacity()) % period_len;
        // return samples;
    }
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;

    mod roll_vec {
        use super::*;

        #[test]
        fn single_complete_copy() {
            let original = vec![1, 2, 3];
            let rolled = roll_vec(&original, 0, 3);
            assert_eq!(rolled, original);
        }

        #[test]
        fn with_head() {
            let original = vec![1, 2, 3];
            let rolled = roll_vec(&original, 2, 4);
            assert_eq!(rolled, [3, 1, 2, 3]);
        }

        #[test]
        fn with_head_body_and_tail() {
            let original = vec![1, 2, 3];
            let rolled = roll_vec(&original, 1, 4);
            assert_eq!(rolled, [2, 3, 1, 2]);
        }

        #[test]
        fn with_head_body_and_tail_multiple_bodies() {
            let original = vec![1, 2, 3];
            let rolled = roll_vec(&original, 1, 9);
            assert_eq!(rolled, [2, 3, 1, 2, 3, 1, 2, 3, 1]);
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
            // 250_000_000 / s
            assert!(false);
        }
    }
}
