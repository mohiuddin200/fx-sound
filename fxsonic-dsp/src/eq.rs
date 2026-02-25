// 10-Band Parametric EQ
// Uses manual biquad filters for peaking EQ (simple f32 API)

use crate::biquad::Biquad;

pub struct EQBand {
    filter: Biquad,
    frequency: f32,
    gain: f32,     // dB, -12 to +12
    q: f32,
    sample_rate: f32,
}

impl EQBand {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let mut filter = Biquad::new();
        filter.set_bell(frequency, 2.0, 0.0, sample_rate);
        Self {
            filter,
            frequency,
            gain: 0.0,
            q: 2.0,
            sample_rate,
        }
    }

    pub fn set_gain(&mut self, gain_db: f32) {
        self.gain = gain_db.clamp(-12.0, 12.0);
        self.update_coefficients();
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q.clamp(1.0, 10.0);
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        self.filter.set_bell(self.frequency, self.q, self.gain, self.sample_rate);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.filter.process(input)
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }
}

pub struct EQProcessor {
    bands: [EQBand; 10],
    sample_rate: f32,
    bypassed: bool,
}

impl EQProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let frequencies = [31.0, 62.0, 125.0, 250.0, 500.0,
                          1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
        let bands = frequencies.map(|f| EQBand::new(f, sample_rate));

        Self {
            bands,
            sample_rate,
            bypassed: false,
        }
    }

    pub fn set_band_gain(&mut self, band_index: usize, gain_db: f32) {
        if band_index < 10 {
            self.bands[band_index].set_gain(gain_db);
        }
    }

    pub fn set_all_gains(&mut self, gains: &[f32; 10]) {
        for (i, &gain) in gains.iter().enumerate() {
            self.bands[i].set_gain(gain);
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.bypassed {
            return input;
        }

        let mut output = input;
        for band in &mut self.bands {
            output = band.process(output);
        }
        output
    }

    pub fn set_bypass(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn get_band_frequency(&self, index: usize) -> Option<f32> {
        self.bands.get(index).map(|b| b.get_frequency())
    }

    pub fn get_band_gain(&self, index: usize) -> Option<f32> {
        self.bands.get(index).map(|b| b.get_gain())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_band_zero_gain() {
        let mut band = EQBand::new(1000.0, 48000.0);

        // Process a signal with zero gain
        let input = 0.5;
        let output = band.process(input);

        // Should be close to input (may differ slightly due to filter)
        assert!((output - input).abs() < 0.1);
    }

    #[test]
    fn test_eq_band_clamping() {
        let mut band = EQBand::new(1000.0, 48000.0);

        // Set gain beyond limits
        band.set_gain(20.0);  // Should clamp to 12
        assert_eq!(band.get_gain(), 12.0);

        band.set_gain(-20.0); // Should clamp to -12
        assert_eq!(band.get_gain(), -12.0);
    }

    #[test]
    fn test_eq_processor_all_bands() {
        let mut processor = EQProcessor::new(48000.0);

        // Set gains for all bands
        let gains: [f32; 10] = [3.0, 2.0, 1.0, 0.0, -1.0, -2.0, 0.0, 2.0, 3.0, 4.0];
        processor.set_all_gains(&gains);

        // Verify gains were set
        for i in 0..10 {
            assert_eq!(processor.get_band_gain(i).unwrap(), gains[i]);
        }
    }

    #[test]
    fn test_eq_bypass() {
        let mut processor = EQProcessor::new(48000.0);
        processor.set_bypass(true);

        let input = 0.5;
        let output = processor.process(input);

        assert_eq!(output, input);
    }

    #[test]
    fn test_eq_band_frequencies() {
        let processor = EQProcessor::new(48000.0);

        let expected_frequencies = [31.0, 62.0, 125.0, 250.0, 500.0,
                                    1000.0, 2000.0, 4000.0, 8000.0, 16000.0];

        for i in 0..10 {
            assert_eq!(processor.get_band_frequency(i).unwrap(), expected_frequencies[i]);
        }
    }
}
