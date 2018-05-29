pub fn harmonic_series(fundamental: f32, partials: usize) -> Vec<f32> {
    (1..partials + 1).map(|p| p as f32 * fundamental).collect()
}

#[cfg(test)]
mod test_harmonic_series {
    use super::*;
    use test_utils::*;

    #[test]
    fn zero_partials() {
        let expected = Vec::<f32>::new();
        assert_almost_eq_by_element(harmonic_series(1., 0), expected);
    }

    #[test]
    fn one_partial() {
        let expected = vec![1.];
        assert_almost_eq_by_element(harmonic_series(1., 1), expected);
    }

    #[test]
    fn multiple_partials() {
        let expected = vec![1., 2., 3.];
        assert_almost_eq_by_element(harmonic_series(1., 3), expected);
    }

    #[test]
    fn multiple_partials_from_a440() {
        let expected = vec![440., 880., 1320., 1760.];
        assert_almost_eq_by_element(harmonic_series(440., 4), expected);
    }
}
