// Clarity - Dynamic high-frequency enhancement for crisp, detailed sound
// Uses manual high-shelf filter (biquad) + envelope follower for dynamic boost

use crate::biquad::Biquad;

pub struct ClarityProcessor {
    high_shelf: Biquad,
    envelope: f32,  // Simple envelope follower state
    amount: f32,
    frequency: f32,
    sample_rate: f32,
    attack: f32,
    release: f32,
}

impl ClarityProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let mut high_shelf = Biquad::new();
        high_shelf.set_highshelf(4000.0, 0.707, 0.0, sample_rate);

        Self {
            high_shelf,
            envelope: 0.0,
            amount: 0.0,
            frequency: 4000.0,
            sample_rate,
            attack: 0.005,
            release: 0.1,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
        self.update_filters();
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(2000.0, 10000.0);
        self.update_filters();
    }

    fn update_filters(&mut self) {
        let gain_db = self.amount * 8.0; // 0 to 8 dB
        self.high_shelf.set_highshelf(self.frequency, 0.707, gain_db, self.sample_rate);
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Track envelope (simple leaky integrator)
        let input_level = input.abs();
        let alpha = if input_level > self.envelope {
            1.0 - (-1.0 / (self.sample_rate * self.attack)).exp()  // Attack
        } else {
            1.0 - (-1.0 / (self.sample_rate * self.release)).exp()  // Release
        };
        self.envelope = self.envelope * (1.0 - alpha) + input_level * alpha;

        // Dynamic boost: more gain when signal has high-frequency content
        let dynamic_boost = self.envelope * self.amount * 6.0; // up to 6 dB extra

        // Apply high-shelf with dynamic boost
        let processed = self.high_shelf.process(input);

        // Add dynamic enhancement
        processed * Self::db_to_linear(dynamic_boost)
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }
}

impl Default for ClarityProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarity_zero_amount() {
        let mut processor = ClarityProcessor::new(48000.0);
        processor.set_amount(0.0);

        let input = 0.5;
        let output = processor.process(input);

        // With zero amount, should be close to passthrough
        assert!((output - input).abs() < 0.1);
    }

    #[test]
    fn test_clarity_with_amount() {
        let mut processor = ClarityProcessor::new(48000.0);
        processor.set_amount(0.5);

        let input = 0.3;
        let output = processor.process(input);

        // Output should be different (amplified/processed)
        // Allow some tolerance due to envelope follower state
        assert!((output - input).abs() > 0.001);
    }

    #[test]
    fn test_clarity_clamping() {
        let mut processor = ClarityProcessor::new(48000.0);

        processor.set_amount(1.5);
        assert_eq!(processor.get_amount(), 1.0);

        processor.set_amount(-0.5);
        assert_eq!(processor.get_amount(), 0.0);
    }

    #[test]
    fn test_clarity_frequency_clamping() {
        let mut processor = ClarityProcessor::new(48000.0);

        processor.set_frequency(1000.0);
        assert_eq!(processor.get_frequency(), 2000.0);

        processor.set_frequency(15000.0);
        assert_eq!(processor.get_frequency(), 10000.0);
    }

    #[test]
    fn test_db_to_linear() {
        // 0 dB = 1.0 linear
        assert!((ClarityProcessor::db_to_linear(0.0) - 1.0).abs() < 0.001);
        // 6 dB ≈ 2.0 linear
        assert!((ClarityProcessor::db_to_linear(6.0) - 2.0).abs() < 0.01);
        // -6 dB ≈ 0.5 linear
        assert!((ClarityProcessor::db_to_linear(-6.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_clarity_frequency_default() {
        let processor = ClarityProcessor::new(48000.0);
        assert_eq!(processor.get_frequency(), 4000.0);
    }

    #[test]
    fn test_clarity_envelope_tracking() {
        let mut processor = ClarityProcessor::new(48000.0);
        processor.set_amount(0.5);

        // Process multiple samples to let envelope settle
        let mut output = 0.0;
        for _ in 0..100 {
            output = processor.process(0.8);
        }

        // Envelope should have settled close to the input level
        assert!((output - 0.8).abs() < 0.3);
    }
}
