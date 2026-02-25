// Main Audio Processor - Orchestrates all effect modules
// Processing chain: Bass Boost → Clarity → Surround → Dynamic Boost → EQ → Ambiance → Limiter

use crate::limiter::LimiterProcessor;
use crate::eq::EQProcessor;
use crate::bass_boost::BassBoostProcessor;
use crate::clarity::ClarityProcessor;
use crate::dynamic_boost::DynamicBoostProcessor;
use crate::surround::SurroundProcessor;
use crate::ambiance::AmbianceProcessor;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioParams {
    pub enabled: bool,

    // Effect amounts (0.0 to 1.0)
    pub bass_boost: f32,
    pub clarity: f32,
    pub ambiance: f32,
    pub surround: f32,
    pub dynamic_boost: f32,

    // EQ bands (10 bands, -12 to +12 dB)
    pub eq_gains: [f32; 10],
}

impl AudioParams {
    pub fn new() -> Self {
        Self {
            enabled: false,
            bass_boost: 0.0,
            clarity: 0.0,
            ambiance: 0.0,
            surround: 0.0,
            dynamic_boost: 0.0,
            eq_gains: [0.0; 10],
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            enabled: true,
            bass_boost: 0.5,
            clarity: 0.5,
            ambiance: 0.3,
            surround: 0.4,
            dynamic_boost: 0.5,
            eq_gains: [0.0; 10],
        }
    }
}

pub struct AudioProcessor {
    params: AudioParams,
    sample_rate: f32,

    // Effect processors
    limiter: LimiterProcessor,
    eq: EQProcessor,
    bass_boost: BassBoostProcessor,
    clarity: ClarityProcessor,
    dynamic_boost: DynamicBoostProcessor,
    surround: SurroundProcessor,
    ambiance: AmbianceProcessor,
}

impl AudioProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            params: AudioParams::new(),
            sample_rate,
            limiter: LimiterProcessor::new(),
            eq: EQProcessor::new(sample_rate),
            bass_boost: BassBoostProcessor::new(sample_rate),
            clarity: ClarityProcessor::new(sample_rate),
            dynamic_boost: DynamicBoostProcessor::new(sample_rate),
            surround: SurroundProcessor::new(sample_rate),
            ambiance: AmbianceProcessor::new(sample_rate),
        }
    }

    pub fn set_params(&mut self, params: AudioParams) {
        self.params = params;

        // Update individual processors
        self.bass_boost.set_amount(params.bass_boost);
        self.clarity.set_amount(params.clarity);
        self.ambiance.set_amount(params.ambiance);
        self.surround.set_amount(params.surround);
        self.dynamic_boost.set_amount(params.dynamic_boost);

        // Update EQ bands
        for (i, &gain) in params.eq_gains.iter().enumerate() {
            self.eq.set_band_gain(i, gain);
        }
    }

    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        if !self.params.enabled {
            // Bypass: just apply limiter for safety
            self.limiter.process(left, right);
            return;
        }

        // Process each sample pair
        for i in 0..left.len() {
            let mut l = left[i];
            let mut r = right[i];

            // 1. Bass Boost
            l = self.bass_boost.process(l);
            r = self.bass_boost.process(r);

            // 2. Clarity
            l = self.clarity.process(l);
            r = self.clarity.process(r);

            // 3. Surround Sound
            let (l2, r2) = self.surround.process_stereo(l, r);
            l = l2;
            r = r2;

            // 4. Dynamic Boost
            l = self.dynamic_boost.process(l);
            r = self.dynamic_boost.process(r);

            // 5. EQ
            l = self.eq.process(l);
            r = self.eq.process(r);

            // 6. Ambiance (after EQ to keep reverb clean)
            let (l2, r2) = self.ambiance.process_stereo(l, r);
            l = l2;
            r = r2;

            left[i] = l;
            right[i] = r;
        }

        // 7. Limiter (always last)
        self.limiter.process(left, right);
    }

    pub fn get_params(&self) -> AudioParams {
        self.params
    }

    pub fn is_enabled(&self) -> bool {
        self.params.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.params.enabled = enabled;
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn get_eq_frequency(&self, band_index: usize) -> Option<f32> {
        self.eq.get_band_frequency(band_index)
    }

    pub fn get_eq_gain(&self, band_index: usize) -> Option<f32> {
        self.eq.get_band_gain(band_index)
    }

    pub fn set_eq_band_gain(&mut self, band_index: usize, gain: f32) {
        self.eq.set_band_gain(band_index, gain);
        self.params.eq_gains[band_index] = gain;
    }

    pub fn set_effect_amount(&mut self, effect: EffectType, amount: f32) {
        match effect {
            EffectType::BassBoost => {
                self.bass_boost.set_amount(amount);
                self.params.bass_boost = amount;
            }
            EffectType::Clarity => {
                self.clarity.set_amount(amount);
                self.params.clarity = amount;
            }
            EffectType::Ambiance => {
                self.ambiance.set_amount(amount);
                self.params.ambiance = amount;
            }
            EffectType::Surround => {
                self.surround.set_amount(amount);
                self.params.surround = amount;
            }
            EffectType::DynamicBoost => {
                self.dynamic_boost.set_amount(amount);
                self.params.dynamic_boost = amount;
            }
        }
    }

    pub fn get_effect_amount(&self, effect: EffectType) -> f32 {
        match effect {
            EffectType::BassBoost => self.bass_boost.get_amount(),
            EffectType::Clarity => self.clarity.get_amount(),
            EffectType::Ambiance => self.ambiance.get_amount(),
            EffectType::Surround => self.surround.get_amount(),
            EffectType::DynamicBoost => self.dynamic_boost.get_amount(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    BassBoost,
    Clarity,
    Ambiance,
    Surround,
    DynamicBoost,
}

impl EffectType {
    pub fn all() -> &'static [EffectType] {
        &[
            EffectType::BassBoost,
            EffectType::Clarity,
            EffectType::Ambiance,
            EffectType::Surround,
            EffectType::DynamicBoost,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EffectType::BassBoost => "Bass Boost",
            EffectType::Clarity => "Clarity",
            EffectType::Ambiance => "Ambiance",
            EffectType::Surround => "Surround Sound",
            EffectType::DynamicBoost => "Dynamic Boost",
        }
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_initialization() {
        let processor = AudioProcessor::new(48000.0);
        assert_eq!(processor.get_sample_rate(), 48000.0);
        assert!(!processor.is_enabled());
    }

    #[test]
    fn test_processor_enable_disable() {
        let mut processor = AudioProcessor::new(48000.0);

        assert!(!processor.is_enabled());
        processor.set_enabled(true);
        assert!(processor.is_enabled());
        processor.set_enabled(false);
        assert!(!processor.is_enabled());
    }

    #[test]
    fn test_processor_set_params() {
        let mut processor = AudioProcessor::new(48000.0);

        let params = AudioParams::with_defaults();
        processor.set_params(params);

        assert!(processor.is_enabled());
        assert_eq!(processor.get_effect_amount(EffectType::BassBoost), 0.5);
        assert_eq!(processor.get_effect_amount(EffectType::Clarity), 0.5);
    }

    #[test]
    fn test_processor_effect_amounts() {
        let mut processor = AudioProcessor::new(48000.0);

        processor.set_effect_amount(EffectType::BassBoost, 0.75);
        assert_eq!(processor.get_effect_amount(EffectType::BassBoost), 0.75);
        assert_eq!(processor.get_params().bass_boost, 0.75);

        processor.set_effect_amount(EffectType::Clarity, 0.25);
        assert_eq!(processor.get_effect_amount(EffectType::Clarity), 0.25);
    }

    #[test]
    fn test_processor_eq_bands() {
        let mut processor = AudioProcessor::new(48000.0);

        processor.set_eq_band_gain(0, 6.0);
        assert_eq!(processor.get_eq_gain(0).unwrap(), 6.0);
        assert_eq!(processor.get_params().eq_gains[0], 6.0);

        processor.set_eq_band_gain(5, -3.0);
        assert_eq!(processor.get_eq_gain(5).unwrap(), -3.0);

        let freq = processor.get_eq_frequency(5);
        assert!(freq.is_some());
    }

    #[test]
    fn test_processor_bypass() {
        let mut processor = AudioProcessor::new(48000.0);

        let params = AudioParams::with_defaults();
        processor.set_params(params);

        // Disable and process
        processor.set_enabled(false);
        let mut left = vec![0.5f32; 512];
        let mut right = vec![0.5f32; 512];

        processor.process_stereo(&mut left, &mut right);

        // Should still process through limiter
        assert!(left.iter().all(|&x| x.abs() <= 1.0));
        assert!(right.iter().all(|&x| x.abs() <= 1.0));
    }

    #[test]
    fn test_processor_full_chain() {
        let mut processor = AudioProcessor::new(48000.0);

        let params = AudioParams {
            enabled: true,
            bass_boost: 0.5,
            clarity: 0.5,
            ambiance: 0.3,
            surround: 0.4,
            dynamic_boost: 0.5,
            eq_gains: [3.0, 2.0, 1.0, 0.0, -1.0, -2.0, 0.0, 2.0, 3.0, 4.0],
        };
        processor.set_params(params);

        // Process a buffer
        let mut left = vec![0.3f32; 512];
        let mut right = vec![0.3f32; 512];

        processor.process_stereo(&mut left, &mut right);

        // Verify no clipping (limiter should have prevented it)
        let max_sample = left.iter()
            .chain(right.iter())
            .fold(0.0f32, |acc, &x| acc.max(x.abs()));

        assert!(max_sample <= 0.8, "Output clipped! Max: {}", max_sample);
    }

    #[test]
    fn test_effect_type_names() {
        assert_eq!(EffectType::BassBoost.name(), "Bass Boost");
        assert_eq!(EffectType::Clarity.name(), "Clarity");
        assert_eq!(EffectType::Ambiance.name(), "Ambiance");
        assert_eq!(EffectType::Surround.name(), "Surround Sound");
        assert_eq!(EffectType::DynamicBoost.name(), "Dynamic Boost");
    }

    #[test]
    fn test_effect_type_all() {
        let all = EffectType::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&EffectType::BassBoost));
        assert!(all.contains(&EffectType::DynamicBoost));
    }

    #[test]
    fn test_audio_params_default() {
        let params = AudioParams::default();
        assert!(!params.enabled);
        assert_eq!(params.bass_boost, 0.0);
        assert_eq!(params.clarity, 0.0);
        assert!(params.eq_gains.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_audio_params_with_defaults() {
        let params = AudioParams::with_defaults();
        assert!(params.enabled);
        assert_eq!(params.bass_boost, 0.5);
        assert_eq!(params.clarity, 0.5);
        assert_eq!(params.ambiance, 0.3);
        assert_eq!(params.surround, 0.4);
        assert_eq!(params.dynamic_boost, 0.5);
    }
}
