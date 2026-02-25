// Dynamic Boost - 3-band multiband compressor
// Splits audio into low/mid/high bands, compresses each, then re-sums

use crate::biquad::Biquad;

pub struct BandCompressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup_gain: f32,
    envelope: f32,
}

impl BandCompressor {
    pub fn new(_sample_rate: f32) -> Self {
        Self {
            threshold: 0.1,  // -20 dBFS
            ratio: 2.0,
            attack: 0.01,   // 10ms
            release: 0.1,   // 100ms
            makeup_gain: 1.5, // +3.5 dB makeup
            envelope: 0.0,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 10.0);
    }

    pub fn process(&mut self, input: f32, dt: f32) -> f32 {
        let input_level = input.abs();

        // Envelope follower (simple leaky integrator)
        let alpha = if input_level > self.envelope {
            dt / self.attack  // Attack
        } else {
            dt / self.release // Release
        };
        self.envelope = self.envelope * (1.0 - alpha) + input_level * alpha;

        // Calculate gain reduction
        let over_threshold = if self.envelope > self.threshold {
            (self.envelope / self.threshold - 1.0) / self.ratio
        } else {
            0.0
        };

        let gain_reduction = 1.0 / (1.0 + over_threshold);
        let output = input * gain_reduction;

        // Auto makeup gain
        output * self.makeup_gain
    }

    pub fn set_makeup_gain(&mut self, gain: f32) {
        self.makeup_gain = gain.clamp(1.0, 4.0);
    }
}

pub struct DynamicBoostProcessor {
    low_band: BandCompressor,
    mid_band: BandCompressor,
    high_band: BandCompressor,
    // Crossover filters (using lowpass/highpass for Linkwitz-Riley-like response)
    low_mid_lowpass: Biquad,
    low_mid_highpass: Biquad,
    mid_high_lowpass: Biquad,
    mid_high_highpass: Biquad,
    amount: f32,
    sample_rate: f32,
    crossover_low_mid: f32,
    crossover_mid_high: f32,
}

impl DynamicBoostProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let low_mid = 250.0;
        let mid_high = 4000.0;

        let mut low_mid_lowpass = Biquad::new();
        low_mid_lowpass.set_lowpass(low_mid, 0.707, sample_rate);

        let mut low_mid_highpass = Biquad::new();
        low_mid_highpass.set_highpass(low_mid, 0.707, sample_rate);

        let mut mid_high_lowpass = Biquad::new();
        mid_high_lowpass.set_lowpass(mid_high, 0.707, sample_rate);

        let mut mid_high_highpass = Biquad::new();
        mid_high_highpass.set_highpass(mid_high, 0.707, sample_rate);

        Self {
            low_band: BandCompressor::new(sample_rate),
            mid_band: BandCompressor::new(sample_rate),
            high_band: BandCompressor::new(sample_rate),
            low_mid_lowpass,
            low_mid_highpass,
            mid_high_lowpass,
            mid_high_highpass,
            amount: 0.0,
            sample_rate,
            crossover_low_mid: low_mid,
            crossover_mid_high: mid_high,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
        let ratio = 2.0 + (self.amount * 2.0); // 2:1 to 4:1
        self.low_band.set_ratio(ratio);
        self.mid_band.set_ratio(ratio);
        self.high_band.set_ratio(ratio);
    }

    pub fn set_crossovers(&mut self, low_mid: f32, mid_high: f32) {
        self.crossover_low_mid = low_mid.clamp(100.0, 500.0);
        self.crossover_mid_high = mid_high.clamp(2000.0, 8000.0);

        self.low_mid_lowpass.set_lowpass(self.crossover_low_mid, 0.707, self.sample_rate);
        self.low_mid_highpass.set_highpass(self.crossover_low_mid, 0.707, self.sample_rate);
        self.mid_high_lowpass.set_lowpass(self.crossover_mid_high, 0.707, self.sample_rate);
        self.mid_high_highpass.set_highpass(self.crossover_mid_high, 0.707, self.sample_rate);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Split into 3 bands
        let low_mid_lp = self.low_mid_lowpass.process(input);
        let low_mid_hp = self.low_mid_highpass.process(input);
        let mid = self.mid_high_lowpass.process(low_mid_hp);
        let high = self.mid_high_highpass.process(low_mid_hp);

        // Compress each band
        let dt = 1.0 / self.sample_rate;
        let low_compressed = self.low_band.process(low_mid_lp, dt);
        let mid_compressed = self.mid_band.process(mid, dt);
        let high_compressed = self.high_band.process(high, dt);

        // Re-sum bands
        low_compressed + mid_compressed + high_compressed
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }
}

impl Default for DynamicBoostProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_boost_zero_amount() {
        let mut processor = DynamicBoostProcessor::new(48000.0);
        processor.set_amount(0.0);

        let input = 0.5;
        let output = processor.process(input);

        // With zero amount, should be passthrough
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_band_compressor_ratio() {
        let mut compressor = BandCompressor::new(48000.0);

        compressor.set_ratio(4.0);
        assert_eq!(compressor.ratio, 4.0);

        compressor.set_ratio(15.0);
        assert_eq!(compressor.ratio, 10.0); // Should clamp

        compressor.set_ratio(0.5);
        assert_eq!(compressor.ratio, 1.0); // Should clamp
    }

    #[test]
    fn test_dynamic_boost_with_amount() {
        let mut processor = DynamicBoostProcessor::new(48000.0);
        processor.set_amount(0.5);

        let input = 0.5;
        let output = processor.process(input);

        // Output should be different (compressed + makeup)
        assert!((output - input).abs() > 0.01);
    }

    #[test]
    fn test_dynamic_boost_clamping() {
        let mut processor = DynamicBoostProcessor::new(48000.0);

        processor.set_amount(1.5);
        assert_eq!(processor.get_amount(), 1.0);

        processor.set_amount(-0.5);
        assert_eq!(processor.get_amount(), 0.0);
    }

    #[test]
    fn test_crossover_clamping() {
        let mut processor = DynamicBoostProcessor::new(48000.0);

        processor.set_crossovers(50.0, 1000.0);
        assert_eq!(processor.crossover_low_mid, 100.0);
        assert_eq!(processor.crossover_mid_high, 2000.0);

        processor.set_crossovers(600.0, 10000.0);
        assert_eq!(processor.crossover_low_mid, 500.0);
        assert_eq!(processor.crossover_mid_high, 8000.0);
    }

    #[test]
    fn test_compressor_envelope() {
        let mut compressor = BandCompressor::new(48000.0);

        let dt = 1.0 / 48000.0;
        let _output1 = compressor.process(0.5, dt);
        let _output2 = compressor.process(0.5, dt);

        // Envelope should have moved towards 0.5
        assert!(compressor.envelope > 0.0);

        // Process many samples to let envelope settle
        for _ in 0..1000 {
            compressor.process(0.5, dt);
        }

        // Envelope should be close to input (within 20% tolerance)
        // (Envelope follower may not settle exactly due to attack/release times)
        assert!((compressor.envelope - 0.5).abs() < 0.2);
    }

    #[test]
    fn test_default_crossovers() {
        let processor = DynamicBoostProcessor::new(48000.0);
        assert_eq!(processor.crossover_low_mid, 250.0);
        assert_eq!(processor.crossover_mid_high, 4000.0);
    }
}
