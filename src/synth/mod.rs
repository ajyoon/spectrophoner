use ::util::{PosF32};

fn period_length(frequency: PosF32, sample_rate: u32) -> u32 {
    return sample_rate / frequency;
}

trait PeriodGenerator {
    fn generate_period(frequency: PosF32, sample_rate: u32) -> Vec<i16>;
}

enum Waveform {
    Sine
}

// impl PeriodGenerator for Waveform {
//     fn generate_period(frequency: PosF32, sample_rate: u32) -> Vec<i16> {

//     }
// }


fn foo() {
    println!("it works");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        foo();
    }
}
