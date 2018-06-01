use image::Rgb;

/// The maximum value our weighted euclidian distance method can produce.
/// This is used to scale results within a 0-1 bound.
///
/// Note that the _actual_ value is more like `764.834` but we leave a small
/// margin to account for rounding errors so that the output value should never
/// be greater than 1.0;
const MAX_UNSCALED_DISTANCE: f32 = 764.835;

/// Calculate a perceptual distance between two RGB colors
///
/// Uses a weighted euclidean distance approximation courtesy of compuphase:
/// https://www.compuphase.com/cmetric.htm
///
/// The resulting distance will be an absolute value between 0 and 1,
/// where 0 is close and 1 is far.
pub fn color_distance(left: Rgb<u8>, right: Rgb<u8>) -> f32 {
    let r_mean = (left.data[0] as f32 + right.data[0] as f32) / 2.0;
    let r = left.data[0] as f32 - right.data[0] as f32;
    let g = left.data[1] as f32 - right.data[1] as f32;
    let b = left.data[2] as f32 - right.data[2] as f32;

    let unscaled = (((2.0 + (r_mean / 256.0)) * r.powi(2))
        + (4.0 * g.powi(2))
        + ((2.0 + ((255.0 - r_mean) / 256.0)) * b.powi(2)))
        .sqrt();
    let scaled = unscaled / MAX_UNSCALED_DISTANCE;
    debug_assert!(scaled <= 1.0);
    return scaled;
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use itertools::Itertools;
    use test_utils::*;
    use std::ops::Range;

    #[test]
    fn test_zero_distance() {
        assert_almost_eq(color_distance(rgb(10, 15, 20), rgb(10, 15, 20)), 0.);
    }

    #[test]
    fn test_black_and_white() {
        assert_almost_eq(color_distance(rgb(0, 0, 0), rgb(255, 255, 255)), 1.);
    }

    #[bench]
    fn color_distances(b: &mut test::Bencher) {
        // ~10.5m crunches per second on shitty linux box
        b.iter(move || {
            for channels in (0..6).map(|_| 2..=255)
                .multi_cartesian_product()
                .take(100_000)
            {
                test::black_box(color_distance(
                    rgb(channels[0], channels[1], channels[2]),
                    rgb(channels[3], channels[4], channels[5]),
                ));
            }
        });
    }

    fn rgb(r: u8, g: u8, b: u8) -> Rgb<u8> {
        Rgb { data: [r, g, b] }
    }
}
