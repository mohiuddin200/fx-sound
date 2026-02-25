// Bass Boost - Enhances low frequencies with warmth and harmonic richness
// Uses manual low-shelf filter (biquad) + harmonic exciter (tanh waveshaper)

use crate::biquad::Biquad;

pub struct BassBoostProcessor {
    low_shelf: Biquad,
    amount: f32,
    frequency: f32,
    sample_rate: f32,
}

impl BassBoostProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let mut low_shelf = Biquad::new();
        low_shelf.set_lowshelf(100.0, 0.707, 0.0, sample_rate);

        Self {
            low_shelf,
            amount: 0.0,
            frequency: 100.0,
            sample_rate,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
        self.update_filters();
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(50.0, 250.0);
        self.update_filters();
    }

    fn update_filters(&mut self) {
        // Map amount 0-1 to gain 0-12 dB
        let gain_db = self.amount * 12.0;
        self.low_shelf.set_lowshelf(self.frequency, 0.707, gain_db, self.sample_rate);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Apply low-shelf boost
        let boosted = self.low_shelf.process(input);

        // Blend in harmonic exciter using tanh waveshaper
        // Extract high-frequency harmonics and boost them
        let harmonics = (boosted - input).tanh();

        // Mix based on amount (higher amount = more exciter)
        boosted + (harmonics * self.amount * 0.5)
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }
}

impl Default for BassBoostProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bass_boost_zero_amount() {
        let mut processor = BassBoostProcessor::new(48000.0);
        processor.set_amount(0.0);

        let input = 0.5;
        let output = processor.process(input);

        // With zero amount, should be passthrough
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_bass_boost_with_amount() {
        let mut processor = BassBoostProcessor::new(48000.0);
        processor.set_amount(0.5);

        let input = 0.3;
        let output = processor.process(input);

        // Output should be different from input due to harmonic exciter
        // Note: low shelf may not boost DC much, but harmonics should add some difference
        // Process multiple samples to build up filter state
        let mut result = output;
        for _ in 0..10 {
            result = processor.process(input);
        }
        assert!((result - input).abs() > 0.001);
    }

    #[test]
    fn test_bass_boost_clamping() {
        let mut processor = BassBoostProcessor::new(48000.0);

        processor.set_amount(1.5);
        assert_eq!(processor.get_amount(), 1.0);

        processor.set_amount(-0.5);
        assert_eq!(processor.get_amount(), 0.0);
    }

    #[test]
    fn test_bass_boost_frequency_clamping() {
        let mut processor = BassBoostProcessor::new(48000.0);

        processor.set_frequency(20.0);
        assert_eq!(processor.get_frequency(), 50.0);

        processor.set_frequency(500.0);
        assert_eq!(processor.get_frequency(), 250.0);
    }

    #[test]
    fn test_bass_boost_frequency_default() {
        let processor = BassBoostProcessor::new(48000.0);
        assert_eq!(processor.get_frequency(), 100.0);
    }
}
