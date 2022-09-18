/// Generate a linear chirp digital signal.
/// The instantaneous frequency change from `freq_a` to `freq_b`.
/// The signal contains exactly `len` samples and the sampling rate is `sample_rate`
pub fn chirp(
    freq_a: f32,
    freq_b: f32,
    len: usize,
    sample_rate: usize,
) -> impl ExactSizeIterator<Item = f32> {
    let dt = 1.0 / sample_rate as f32;
    let duration = dt * len as f32;
    let df_dt = (freq_b - freq_a) / duration;

    use std::f32::consts::{PI, TAU};
    // delta phase / delta time = 2*pi*freq_a + 2*pi*df_dt*t
    (0..len).map(move |i| {
        let t = i as f32 * dt;
        let phase = TAU * freq_a * t + PI * df_dt * t * t;
        phase.sin()
    })
}

/// Compute the dot product of two sequences
pub fn dot_product<'a, 'b, Ia, Ib>(seq_a: Ia, seq_b: Ib) -> f32
where
    Ia: ExactSizeIterator<Item = &'a f32>,
    Ib: ExactSizeIterator<Item = &'a f32>,
{
    assert_eq!(seq_a.len(), seq_b.len());
    seq_a.zip(seq_b).fold(0.0, |sum, (x, y)| sum + x * y)
}

/// Copy samples from `src` to fill `dest`.
/// Return the number of copied samples.
pub fn copy<'a, T, D, S>(dest: D, src: S) -> usize
where
    T: 'a + Clone,
    D: Iterator<Item = &'a mut T>,
    S: Iterator<Item = T>,
{
    dest.zip(src).fold(0, |n, (x, y)| {
        *x = y;
        n + 1
    })
}
