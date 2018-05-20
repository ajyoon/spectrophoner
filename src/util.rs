use std::ops::Div;


/// A 32-bit floating number which is above zero.
pub struct PosF32(f32);

impl PosF32 {
    pub fn new(val: f32) -> PosF32 {
        if val > 0. {
            return PosF32(val);
        } else {
            panic!("Value must be above zero");
        }
    }
}

impl Div<u32> for PosF32 {
    type Output = PosF32;
    fn div(self, rhs: u32) -> PosF32 {
        return PosF32::new(self.0 / (rhs as f32));
    }
}

impl Div<PosF32> for u32 {
    type Output = u32;
    fn div(self, rhs: PosF32) -> u32 {
        self / (rhs.0 as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn panics_at_zero() {
        PosF32::new(0.);
    }

    #[test]
    #[should_panic]
    fn panics_below_zero() {
        PosF32::new(-1.);
    }

    #[test]
    fn ok_above_zero() {
        PosF32::new(1.);
    }
}
