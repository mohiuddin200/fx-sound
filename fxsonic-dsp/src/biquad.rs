// Manual Biquad Filter Implementation
// Based on Audio EQ Cookbook formulas by Robert Bristow-Johnson
// https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html

use std::f32::consts::PI;

/// Biquad filter state and coefficients
pub struct Biquad {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // State variables (previous samples)
    x1: f32,  // Previous input
    x2: f32,  // Two samples ago input
    y1: f32,  // Previous output
    y2: f32,  // Two samples ago output
}

impl Biquad {
    /// Create new zero-state biquad filter
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process single sample (Direct Form II)
    pub fn process(&mut self, x: f32) -> f32 {
        // y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
               - self.a1 * self.y1 - self.a2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;

        y
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Set bell/peaking EQ coefficients
    /// freq: center frequency (Hz)
    /// q: bandwidth factor (1-10)
    /// gain: gain in dB
    /// sample_rate: audio sample rate (Hz)
    pub fn set_bell(&mut self, freq: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10.0_f32.powf(gain / 40.0);  // amplitude
        let omega = 2.0 * PI * freq / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let b0 = 1.0 + alpha * A;
        let b1 = -2.0 * cs;
        let b2 = 1.0 - alpha * A;
        let a0 = 1.0 + alpha / A;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha / A;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    /// Set low-shelf coefficients
    /// freq: shelf frequency (Hz)
    /// q: shelf slope factor (0.5-1.0)
    /// gain: gain in dB
    /// sample_rate: audio sample rate (Hz)
    pub fn set_lowshelf(&mut self, freq: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10.0_f32.powf(gain / 40.0);
        let omega = 2.0 * PI * freq / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);
        let two_sqrt_A_alpha = 2.0 * A.sqrt() * alpha;

        let b0 = A * ((A + 1.0) - (A - 1.0) * cs + two_sqrt_A_alpha);
        let b1 = 2.0 * A * ((A - 1.0) - (A + 1.0) * cs);
        let b2 = A * ((A + 1.0) - (A - 1.0) * cs - two_sqrt_A_alpha);
        let a0 = (A + 1.0) + (A - 1.0) * cs + two_sqrt_A_alpha;
        let a1 = -2.0 * ((A - 1.0) + (A + 1.0) * cs);
        let a2 = (A + 1.0) + (A - 1.0) * cs - two_sqrt_A_alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    /// Set high-shelf coefficients
    /// freq: shelf frequency (Hz)
    /// q: shelf slope factor (0.5-1.0)
    /// gain: gain in dB
    /// sample_rate: audio sample rate (Hz)
    pub fn set_highshelf(&mut self, freq: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10.0_f32.powf(gain / 40.0);
        let omega = 2.0 * PI * freq / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);
        let two_sqrt_A_alpha = 2.0 * A.sqrt() * alpha;

        let b0 = A * ((A + 1.0) + (A - 1.0) * cs + two_sqrt_A_alpha);
        let b1 = -2.0 * A * ((A - 1.0) + (A + 1.0) * cs);
        let b2 = A * ((A + 1.0) + (A - 1.0) * cs - two_sqrt_A_alpha);
        let a0 = (A + 1.0) - (A - 1.0) * cs + two_sqrt_A_alpha;
        let a1 = 2.0 * ((A - 1.0) - (A + 1.0) * cs);
        let a2 = (A + 1.0) - (A - 1.0) * cs - two_sqrt_A_alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    /// Set low-pass filter coefficients
    /// freq: cutoff frequency (Hz)
    /// q: resonance/quality factor
    /// sample_rate: audio sample rate (Hz)
    pub fn set_lowpass(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let b0 = (1.0 - cs) / 2.0;
        let b1 = 1.0 - cs;
        let b2 = (1.0 - cs) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    /// Set high-pass filter coefficients
    /// freq: cutoff frequency (Hz)
    /// q: resonance/quality factor
    /// sample_rate: audio sample rate (Hz)
    pub fn set_highpass(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let b0 = (1.0 + cs) / 2.0;
        let b1 = -(1.0 + cs);
        let b2 = (1.0 + cs) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    /// Normalize coefficients so a0 = 1
    fn normalize(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl Default for Biquad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biquad_creation() {
        let biquad = Biquad::new();
        // Should be passthrough with initial coefficients
        assert_eq!(biquad.b0, 1.0);
        assert_eq!(biquad.b1, 0.0);
        assert_eq!(biquad.b2, 0.0);
    }

    #[test]
    fn test_biquad_bell_zero_gain() {
        let mut biquad = Biquad::new();
        biquad.set_bell(1000.0, 2.0, 0.0, 48000.0);

        // Process a signal
        let input = 0.5;
        let output = biquad.process(input);

        // Should be close to input (filter unity at 0 dB)
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_biquad_lowshelf_zero_gain() {
        let mut biquad = Biquad::new();
        biquad.set_lowshelf(100.0, 0.707, 0.0, 48000.0);

        // Process a signal
        let input = 0.5;
        let output = biquad.process(input);

        // Should be close to input at zero gain
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_biquad_highshelf_zero_gain() {
        let mut biquad = Biquad::new();
        biquad.set_highshelf(4000.0, 0.707, 0.0, 48000.0);

        // Process a signal
        let input = 0.5;
        let output = biquad.process(input);

        // Should be close to input at zero gain
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_biquad_lowpass() {
        let mut biquad = Biquad::new();
        biquad.set_lowpass(1000.0, 0.707, 48000.0);

        // Process a DC signal (should pass through)
        let mut output = 0.0;
        for _ in 0..100 {
            output = biquad.process(1.0);
        }
        // Lowpass should pass DC
        assert!(output.abs() > 0.5);
    }

    #[test]
    fn test_biquad_highpass() {
        let mut biquad = Biquad::new();
        biquad.set_highpass(1000.0, 0.707, 48000.0);

        // Process a DC signal (should block)
        let mut output = 0.0;
        for _ in 0..100 {
            output = biquad.process(1.0);
        }
        // Highpass should block DC
        assert!(output.abs() < 0.1);
    }

    #[test]
    fn test_biquad_reset() {
        let mut biquad = Biquad::new();
        biquad.set_bell(1000.0, 2.0, 6.0, 48000.0);
        biquad.process(0.5);
        biquad.process(0.5);

        biquad.reset();

        // State should be zero
        assert_eq!(biquad.x1, 0.0);
        assert_eq!(biquad.x2, 0.0);
        assert_eq!(biquad.y1, 0.0);
        assert_eq!(biquad.y2, 0.0);
    }

    #[test]
    fn test_biquad_stability() {
        let mut biquad = Biquad::new();
        biquad.set_bell(1000.0, 2.0, 12.0, 48000.0);

        // Process many samples to ensure stability
        let mut max_output = 0.0_f32;
        for i in 0..1000 {
            let input = (i as f32 * 0.001).sin();
            let output = biquad.process(input);
            max_output = max_output.max(output.abs());
        }

        // Filter should be stable (not diverge)
        assert!(max_output.is_finite());
        assert!(max_output < 100.0);
    }

    #[test]
    fn test_biquad_default() {
        let biquad = Biquad::default();
        // Should be passthrough with initial coefficients
        assert_eq!(biquad.b0, 1.0);
        assert_eq!(biquad.b1, 0.0);
        assert_eq!(biquad.b2, 0.0);
    }
}
