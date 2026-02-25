// Surround Sound - Stereo widening for immersive experience
// Uses mid-side encoding/decoding with Haas delay

use fundsp::hacker::*;

pub struct SurroundProcessor {
    side_boost_db: f32,
    amount: f32,
    sample_rate: f32,
    // Haas delay buffer (at 48kHz, 5ms = 240 samples)
    delay_samples: usize,
    delay_buffer: Vec<f32>,
    delay_index: usize,
}

impl SurroundProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let haas_delay_ms = 4.0; // 4ms delay
        let delay_samples = (haas_delay_ms / 1000.0 * sample_rate) as usize;
        let buffer_size = delay_samples + 1;

        Self {
            side_boost_db: 0.0,
            amount: 0.0,
            sample_rate,
            delay_samples,
            delay_buffer: vec![0.0; buffer_size],
            delay_index: 0,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
        // Side boost: 0 dB (amount=0) to +6 dB (amount=1)
        self.side_boost_db = self.amount * 6.0;
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.amount == 0.0 {
            return (left, right);
        }

        // Mid-side encoding
        let mid = (left + right) * 0.5;
        let mut side = (left - right) * 0.5;

        // Apply Haas delay to side channel
        self.delay_buffer[self.delay_index] = side;
        let delayed_side = self.delay_buffer[
            (self.delay_index + self.delay_buffer.len() - self.delay_samples) % self.delay_buffer.len()
        ];
        self.delay_index = (self.delay_index + 1) % self.delay_buffer.len();

        // Boost delayed side channel
        let boost_factor = Self::db_to_linear(self.side_boost_db);
        let side_enhanced = delayed_side * boost_factor;

        // Mid-side decoding
        let output_left = mid + side_enhanced;
        let output_right = mid - side_enhanced;

        (output_left, output_right)
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }

    pub fn get_delay_samples(&self) -> usize {
        self.delay_samples
    }
}

impl Default for SurroundProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surround_zero_amount() {
        let mut processor = SurroundProcessor::new(48000.0);
        processor.set_amount(0.0);

        let left = 0.5;
        let right = 0.3;

        let (output_l, output_r) = processor.process_stereo(left, right);

        // With zero amount, should be passthrough
        assert!((output_l - left).abs() < 0.01);
        assert!((output_r - right).abs() < 0.01);
    }

    #[test]
    fn test_surround_with_amount() {
        let mut processor = SurroundProcessor::new(48000.0);
        processor.set_amount(0.5);

        let left = 0.5;
        let right = 0.3;

        let (output_l, output_r) = processor.process_stereo(left, right);

        // Output should be different (widened)
        assert!((output_l - left).abs() > 0.01 || (output_r - right).abs() > 0.01);
    }

    #[test]
    fn test_surround_clamping() {
        let mut processor = SurroundProcessor::new(48000.0);

        processor.set_amount(1.5);
        assert_eq!(processor.get_amount(), 1.0);

        processor.set_amount(-0.5);
        assert_eq!(processor.get_amount(), 0.0);
    }

    #[test]
    fn test_mid_side_encoding_decoding() {
        // Test that M/S encoding/decoding preserves the signal when no processing
        let mid = 0.4;
        let side = 0.1;

        // Decode
        let left = mid + side;
        let right = mid - side;

        // Re-encode and decode
        let re_mid = (left + right) * 0.5;
        let re_side = (left - right) * 0.5;

        assert!((re_mid - mid).abs() < 0.001);
        assert!((re_side - side).abs() < 0.001);
    }

    #[test]
    fn test_db_to_linear() {
        // 0 dB = 1.0 linear
        assert!((SurroundProcessor::db_to_linear(0.0) - 1.0).abs() < 0.001);
        // 6 dB ≈ 2.0 linear
        assert!((SurroundProcessor::db_to_linear(6.0) - 2.0).abs() < 0.01);
        // -6 dB ≈ 0.5 linear
        assert!((SurroundProcessor::db_to_linear(-6.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_delay_buffer_size() {
        let processor_48k = SurroundProcessor::new(48000.0);
        // 4ms at 48kHz = 192 samples
        assert_eq!(processor_48k.get_delay_samples(), 192);

        let processor_44k = SurroundProcessor::new(44100.0);
        // 4ms at 44.1kHz = ~176 samples
        assert_eq!(processor_44k.get_delay_samples(), 176);
    }

    #[test]
    fn test_surround_stereo_preservation() {
        let mut processor = SurroundProcessor::new(48000.0);
        processor.set_amount(0.5);

        // Mono input (same on both channels)
        let mono = 0.5;
        let (output_l, output_r) = processor.process_stereo(mono, mono);

        // For mono input, mid/side processing should result in equal outputs
        assert!((output_l - output_r).abs() < 0.001);

        // Different stereo inputs
        let left = 0.5;
        let right = 0.3;

        // Process multiple times to fill delay buffer
        let mut out_l = left;
        let mut out_r = right;
        for _ in 0..200 {
            (out_l, out_r) = processor.process_stereo(left, right);
        }

        // Outputs should be different after processing
        assert!((out_l - out_r).abs() > 0.01);
    }
}
