// Output Limiter - Prevents clipping by limiting peak amplitude
// Threshold: Fixed at -0.3 dBFS (0.7 linear)

pub struct LimiterProcessor {
    threshold: f32,
}

impl LimiterProcessor {
    pub fn new() -> Self {
        Self { threshold: 0.7f32 } // -0.3 dBFS = 0.7 linear
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let max_amplitude = left
            .iter()
            .chain(right.iter())
            .fold(0.0f32, |acc, &x| acc.max(x.abs()));

        if max_amplitude > self.threshold {
            let gain = self.threshold / max_amplitude;
            for sample in left.iter_mut() {
                *sample *= gain;
            }
            for sample in right.iter_mut() {
                *sample *= gain;
            }
        }
    }
}

impl Default for LimiterProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_no_clip_passthrough() {
        let mut limiter = LimiterProcessor::new();
        let mut left = vec![0.1f32, 0.2, 0.3];
        let mut right = vec![0.1f32, 0.2, 0.3];

        limiter.process(&mut left, &mut right);

        assert!((left[0] - 0.1).abs() < 0.001);
        assert!((right[2] - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_limiter_prevents_clipping() {
        let mut limiter = LimiterProcessor::new();
        let mut left = vec![0.9f32];
        let mut right = vec![0.8f32];

        limiter.process(&mut left, &mut right);

        // Both should be limited to <= 0.7
        assert!(left[0].abs() <= limiter.threshold + 0.01);
        assert!(right[0].abs() <= limiter.threshold + 0.01);
    }

    #[test]
    fn test_limiter_ratio_preserved() {
        let mut limiter = LimiterProcessor::new();
        let mut left = vec![1.0f32, 0.5];
        let mut right = vec![0.8f32, 0.4];

        limiter.process(&mut left, &mut right);

        // After limiting, L/R ratio should be preserved
        let ratio_before = 1.0 / 0.8;
        let ratio_after = left[0] / right[0];
        assert!((ratio_before - ratio_after).abs() < 0.001);
    }
}
