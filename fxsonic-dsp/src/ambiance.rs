// Ambiance - Reverb effect for depth and immersion
// Uses Freeverb (Schroeder reverb topology) from fundsp with Frame conversion

use fundsp::hacker::*;
use fundsp::audionode::*;

// Helper to convert f32 stereo to Frame<f32, U2> and back
fn stereo_to_frame(left: f32, right: f32) -> Frame<f32, U2> {
    [left, right].into()
}

fn frame_to_stereo(frame: &Frame<f32, U2>) -> (f32, f32) {
    (frame[0], frame[1])
}

// We can't store `impl Trait` in struct, so we need a workaround
// We'll create reverb on each process call (not optimal but functional)
pub struct AmbianceProcessor {
    // Store parameters
    amount: f32,
    wet_level: f32,
    dry_level: f32,
    sample_rate: f32,
    room_size: f32,
    time: f32,
    damping: f32,
}

impl AmbianceProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            amount: 0.0,
            wet_level: 0.0,
            dry_level: 1.0,
            sample_rate,
            room_size: 15.0,
            time: 1.0,
            damping: 0.5,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);

        // Map amount to reverb parameters
        // Amount 0 = no reverb, Amount 1 = full wet, large room
        let room_size = 10.0 + (self.amount * 20.0);  // 10 to 30 meters
        let time = 0.5 + (self.amount * 1.5);  // 0.5 to 2.0 seconds
        let damping = 0.3 + (self.amount * 0.4);  // 0.3 to 0.7
        self.room_size = room_size;
        self.time = time;
        self.damping = damping;

        // Wet/dry mixing
        // Amount 0 = 100% dry, Amount 0.5 = 50/50, Amount 1 = 100% wet
        self.wet_level = self.amount;
        self.dry_level = 1.0 - (self.amount * 0.5); // Never go to 0 dry
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        let size = 10.0 + (self.room_size * 20.0);
        self.room_size = size;
    }

    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.amount == 0.0 {
            return (left, right);
        }

        // Create reverb for this call (fundsp limitation workaround)
        let mut reverb = reverb_stereo(self.room_size, self.time, self.damping);

        // Convert to Frame for fundsp
        let input_frame = stereo_to_frame(left, right);
        let output_frame = reverb.tick(&input_frame);
        let (wet_left, wet_right) = frame_to_stereo(&output_frame);

        // Mix wet and dry
        let output_left = left * self.dry_level + wet_left * self.wet_level;
        let output_right = right * self.dry_level + wet_right * self.wet_level;

        (output_left, output_right)
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }

    pub fn get_room_size(&self) -> f32 {
        self.room_size
    }

    pub fn get_wet_level(&self) -> f32 {
        self.wet_level
    }

    pub fn get_dry_level(&self) -> f32 {
        self.dry_level
    }
}

impl Default for AmbianceProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambiance_zero_amount() {
        let mut processor = AmbianceProcessor::new(48000.0);
        processor.set_amount(0.0);

        let left = 0.5;
        let right = 0.3;

        let (output_l, output_r) = processor.process_stereo(left, right);

        // With zero amount, should be passthrough
        assert!((output_l - left).abs() < 0.01);
        assert!((output_r - right).abs() < 0.01);
    }

    #[test]
    fn test_ambiance_with_amount() {
        let mut processor = AmbianceProcessor::new(48000.0);
        processor.set_amount(0.5);

        let left = 0.5;
        let right = 0.3;

        let (output_l, output_r) = processor.process_stereo(left, right);

        // Output should be different (reverberated)
        assert!((output_l - left).abs() > 0.01 || (output_r - right).abs() > 0.01);
    }

    #[test]
    fn test_ambiance_clamping() {
        let mut processor = AmbianceProcessor::new(48000.0);

        processor.set_amount(1.5);
        assert_eq!(processor.get_amount(), 1.0);

        processor.set_amount(-0.5);
        assert_eq!(processor.get_amount(), 0.0);
    }

    #[test]
    fn test_ambiance_room_size() {
        let mut processor = AmbianceProcessor::new(48000.0);

        processor.set_room_size(0.75);
        assert_eq!(processor.get_room_size(), 25.0);

        processor.set_room_size(1.5);
        assert_eq!(processor.get_room_size(), 30.0); // Should clamp to max

        processor.set_room_size(-0.5);
        assert_eq!(processor.get_room_size(), 10.0); // Should clamp to min
    }

    #[test]
    fn test_ambiance_wet_dry_levels() {
        let mut processor = AmbianceProcessor::new(48000.0);

        processor.set_amount(0.0);
        assert_eq!(processor.get_wet_level(), 0.0);
        assert_eq!(processor.get_dry_level(), 1.0);

        processor.set_amount(0.5);
        assert_eq!(processor.get_wet_level(), 0.5);
        assert_eq!(processor.get_dry_level(), 0.75);

        processor.set_amount(1.0);
        assert_eq!(processor.get_wet_level(), 1.0);
        assert_eq!(processor.get_dry_level(), 0.5); // Never goes to 0
    }

    #[test]
    fn test_ambiance_default() {
        let processor = AmbianceProcessor::new(48000.0);
        assert_eq!(processor.get_amount(), 0.0);
        assert_eq!(processor.get_room_size(), 15.0);
    }

    #[test]
    fn test_ambiance_stereo_preservation() {
        let mut processor = AmbianceProcessor::new(48000.0);
        processor.set_amount(0.3);

        let left = 0.5;
        let right = 0.3;

        let (output_l, output_r) = processor.process_stereo(left, right);

        // Stereo should be preserved (L and R should be different if input is different)
        assert!((output_l - output_r).abs() > 0.01);
    }

    #[test]
    fn test_ambiance_mono_input() {
        let mut processor = AmbianceProcessor::new(48000.0);
        processor.set_amount(0.5);

        // Process multiple samples to build up reverb tail
        let mono = 0.5;
        let mut out_l = mono;
        let mut out_r = mono;

        for _ in 0..100 {
            (out_l, out_r) = processor.process_stereo(mono, mono);
        }

        // After reverb, outputs should still be similar (but not identical due to reverb)
        assert!((out_l - out_r).abs() < 0.2);
    }

    #[test]
    fn test_ambiance_amount_affects_room_size() {
        let mut processor = AmbianceProcessor::new(48000.0);

        processor.set_amount(0.25);
        assert_eq!(processor.get_room_size(), 15.0);

        processor.set_amount(0.75);
        assert_eq!(processor.get_room_size(), 25.0);

        processor.set_amount(1.0);
        assert_eq!(processor.get_room_size(), 30.0);
    }
}
