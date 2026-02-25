# FxSonic DSP Engine Implementation Guide

## Phase 1: Audio Processing Pipeline

**Objective:** Implement all audio effects (Bass Boost, Clarity, Ambiance, Surround Sound, Dynamic Boost) and the 10-band parametric EQ in the LADSPA plugin.

**Duration:** Weeks 4-8 (approximately 4-5 weeks)

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Implementation Order](#implementation-order)
4. [Effect Specifications](#effect-specifications)
5. [Step-by-Step Implementation](#step-by-step-implementation)
6. [Testing Strategy](#testing-strategy)
7. [Performance Benchmarks](#performance-benchmarks)
8. [Verification Checklist](#verification-checklist)

---

## Prerequisites

### Before Starting

1. **System Dependencies Installed**
   - All packages from `INITIALIZATION_STATUS.md` must be installed
   - Verify: `cargo build --workspace` completes successfully

2. **Test Environment Setup**
   - PipeWire running and confirmed working
   - Test audio files ready (sine waves at various frequencies, music tracks)

3. **Development Tools**
   - Audio analysis tools: `pw-top`, `pw-cat`, `ffmpeg`
   - Waveform viewer: Audacity or similar
   - Oscilloscope/FFT analyzer (optional): `raffinity`, `qasconfig`

### Required Test Files

Create a test audio library in `fxsonic-dsp/tests/audio/`:

```
tests/audio/
├── sine-100hz-48k-16bit.wav       # 100Hz sine wave
├── sine-1khz-48k-16bit.wav        # 1kHz sine wave
├── sine-10khz-48k-16bit.wav       # 10kHz sine wave
├── white-noise-48k-16bit.wav      # Full spectrum white noise
├── pink-noise-48k-16bit.wav       # Pink noise for frequency response
├── sweep-20hz-20khz-48k-16bit.wav # Logarithmic frequency sweep
└── test-music-48k-16bit.wav       # Full mix music track
```

Generate test files:
```bash
# Generate sine waves
ffmpeg -f lavfi -i "sine=frequency=100:sample_rate=48000:duration=5" tests/audio/sine-100hz-48k-16bit.wav
ffmpeg -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=5" tests/audio/sine-1khz-48k-16bit.wav
ffmpeg -f lavfi -i "sine=frequency=10000:sample_rate=48000:duration=5" tests/audio/sine-10khz-48k-16bit.wav

# Generate noise
ffmpeg -f lavfi -i "anoisesrc=white:sample_rate=48000:duration=5" tests/audio/white-noise-48k-16bit.wav
ffmpeg -f lavfi -i "anoisesrc=pink:sample_rate=48000:duration=5" tests/audio/pink-noise-48k-16bit.wav

# Generate sweep
ffmpeg -f lavfi -i "sine=frequency=20:sample_rate=48000:duration=5" -af "asetrate=48000*2,atempo=2" tests/audio/sweep-20hz-20khz-48k-16bit.wav
```

---

## Architecture Overview

### DSP Processing Chain

```
Input (stereo, f32)
    ↓
┌─────────────────────────────────────────────────────────────┐
│ AudioProcessor (main struct)                                │
├─────────────────────────────────────────────────────────────┤
│ 1. BassBoostProcessor                                       │
│    - Low-shelf filter (fundsp::lowshelf_hz)                 │
│    - Harmonic exciter (soft saturation)                     │
├─────────────────────────────────────────────────────────────┤
│ 2. ClarityProcessor                                         │
│    - High-shelf filter                                       │
│    - Dynamic boost (envelope follower)                      │
├─────────────────────────────────────────────────────────────┤
│ 3. AmbianceProcessor                                        │
│    - Freeverb reverb (fundsp::reverb_stereo)                │
│    - Wet/dry mixer                                           │
├─────────────────────────────────────────────────────────────┤
│ 4. SurroundProcessor                                        │
│    - Mid-side encoding/decoding                             │
│    - Side channel boost                                      │
│    - Haas delay                                               │
├─────────────────────────────────────────────────────────────┤
│ 5. DynamicBoostProcessor                                     │
│    - 3-band crossover (Linkwitz-Riley)                      │
│    - Compressor per band                                     │
│    - Makeup gain                                             │
├─────────────────────────────────────────────────────────────┤
│ 6. EQProcessor                                               │
│    - 10 x Bell filters (fundsp::bell_hz)                    │
│    - Frequency response query for UI                        │
├─────────────────────────────────────────────────────────────┤
│ 7. LimiterProcessor                                          │
│    - Brick-wall limiter at -0.3dBFS                          │
└─────────────────────────────────────────────────────────────┘
    ↓
Output (stereo, f32)
```

### Parameter Update Mechanism

```
┌─────────────────────────────────────┐
│ Tauri Backend (main thread)        │
│                                     │
│ set_effect("Bass", 0.7)            │
│   → params_tx.send(BassParam {    │
│        boost: 0.7,                 │
│        frequency: 100.0            │
│      })                             │
└─────────────────────────────────────┘
            │
            │ crossbeam-channel
            │ (lock-free ring buffer)
            ↓
┌─────────────────────────────────────┐
│ LADSPA Plugin (RT thread)          │
│                                     │
│ process() {                         │
│   if let Some(param) = params_rx.try_recv() {
│     update_filter_coefficients(); // Lock-free!
│   }                                │
│   process_audio();                 │
│ }                                  │
└─────────────────────────────────────┘
```

---

## Implementation Order

Implement effects in this order to build confidence and simplify debugging:

1. **Output Limiter** (Day 1) - Simple, protects against clipping
2. **10-Band Parametric EQ** (Days 2-4) - Foundation for frequency response visualization
3. **Bass Boost** (Days 5-7) - First "character" effect, most audible
4. **Clarity** (Days 8-10) - Complements bass, uses envelope follower
5. **Dynamic Boost** (Days 11-13) - Multi-band processing, most complex
6. **Surround Sound** (Days 14-15) - Stereo processing, mid-side encoding
7. **Ambiance** (Days 16-17) - Reverb, spatial effect

---

## Effect Specifications

### 1. Output Limiter

**Purpose:** Prevent clipping when multiple effects add gain.

**Parameters:**
- Threshold: Fixed at -0.3 dBFS (hardcoded)

**Algorithm:**
- Simple soft-knee limiter using peak detection
- If any sample exceeds -0.3 dBFS, scale all samples in that buffer

**Code Structure:**
```rust
pub struct LimiterProcessor {
    threshold: f32,
}

impl LimiterProcessor {
    pub fn new() -> Self {
        Self { threshold: 0.7f32 } // -0.3 dBFS = 0.7 linear
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let max_amplitude = left.iter()
            .chain(right.iter())
            .fold(0.0f32, |acc, &x| acc.max(x.abs()));

        if max_amplitude > self.threshold {
            let gain = self.threshold / max_amplitude;
            for sample in left.iter_mut() { *sample *= gain; }
            for sample in right.iter_mut() { *sample *= gain; }
        }
    }
}
```

---

### 2. 10-Band Parametric EQ

**Purpose:** Fine-tune frequency response per user preference.

**Parameters (per band):**
- Frequency: 31, 62, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz (default)
- Gain: ±12 dB (0.0 to 1.0 normalized)
- Q Factor: 1.0 to 10.0 (default: 2.0)

**Algorithm:**
- 10 parallel bell filters (peaking EQ)
- Use `fundsp::bell_hz()` for frequency response queries
- Use `biquad::DirectForm1` for real-time coefficient updates

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct EQBand {
    filter: Bell<f32>,
    frequency: f32,
    gain: f32,     // dB, -12 to +12
    q: f32,
}

impl EQBand {
    pub fn new(frequency: f32) -> Self {
        Self {
            filter: bell_hz(frequency, 0.0, 2.0),
            frequency,
            gain: 0.0,
            q: 2.0,
        }
    }

    pub fn set_gain(&mut self, gain_db: f32) {
        self.gain = gain_db;
        self.filter = bell_hz(self.frequency, gain_db, self.q);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.filter.tick(input)
    }

    pub fn frequency_response(&self, frequencies: &[f32]) -> Vec<f32> {
        // Use fundsp's frequency_response method
        // Returns amplitude in dB at each frequency
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
        let bands = frequencies.map(|f| EQBand::new(f));

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
}
```

---

### 3. Bass Boost

**Purpose:** Enhance low frequencies with warmth and harmonic richness.

**Parameters:**
- Amount: 0.0 to 1.0 (normalized)
- Frequency: ~80-120 Hz (adjustable via control port)
- Character: Clean (low) to Warm (high)

**Algorithm:**
1. Low-shelf filter for basic boost (0 to +12 dB)
2. Harmonic exciter for added harmonics (waveshaper)
3. Subtle saturation for warmth

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct BassBoostProcessor {
    low_shelf: LowShelf<f32>,
    exciter: Waveshaper<f32>,
    amount: f32,
    frequency: f32,
    sample_rate: f32,
}

impl BassBoostProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            low_shelf: lowshelf_hz(100.0, 0.0),
            exciter: tanh(),  // Soft saturation for harmonics
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
        self.frequency = frequency;
        self.update_filters();
    }

    fn update_filters(&mut self) {
        // Map amount 0-1 to gain 0-12 dB
        let gain_db = self.amount * 12.0;
        self.low_shelf = lowshelf_hz(self.frequency, gain_db);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Apply low-shelf boost
        let boosted = self.low_shelf.tick(input);

        // Blend in harmonic exciter
        // Extract high-frequency harmonics and boost them
        let harmonics = self.exciter.tick(boosted - input);

        // Mix based on amount (higher amount = more exciter)
        boosted + (harmonics * self.amount * 0.5)
    }
}
```

---

### 4. Clarity

**Purpose:** Dynamic high-frequency enhancement for crisp, detailed sound.

**Parameters:**
- Amount: 0.0 to 1.0 (normalized)
- Frequency: ~3-5 kHz (adjustable)
- Attack/Release: For envelope follower

**Algorithm:**
1. High-shelf filter for basic treble boost
2. Envelope follower detects high-frequency transients
3. Boost is modulated by envelope (dynamic enhancement)

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct ClarityProcessor {
    high_shelf: HighShelf<f32>,
    envelope: Follower<f32>,
    amount: f32,
    frequency: f32,
    sample_rate: f32,
    // Attack: 5ms, Release: 100ms
    attack: f32,
    release: f32,
}

impl ClarityProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            high_shelf: highshelf_hz(4000.0, 0.0),
            envelope: follow(0.005, 0.1), // 5ms attack, 100ms release
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
        self.frequency = frequency;
        self.update_filters();
    }

    fn update_filters(&mut self) {
        let gain_db = self.amount * 8.0; // 0 to 8 dB
        self.high_shelf = highshelf_hz(self.frequency, gain_db);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Track envelope of high frequencies
        let high_freq_envelope = self.envelope.tick(input.abs());

        // Dynamic boost: more gain when signal has high-frequency content
        let dynamic_boost = high_freq_envelope * self.amount * 6.0; // up to 6 dB extra

        // Apply high-shelf with dynamic boost
        let processed = self.high_shelf.tick(input);

        // Add dynamic enhancement
        processed * db_to_linear(dynamic_boost)
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }
}
```

---

### 5. Dynamic Boost

**Purpose:** 3-band multiband compression restores amplitude lost in compression.

**Parameters:**
- Amount: 0.0 to 1.0 (normalized)
- Crossover frequencies: Low/Mid @ 250 Hz, Mid/High @ 4 kHz (adjustable)
- Threshold: -20 dB (fixed)
- Ratio: 2:1 to 4:1 (based on amount)
- Attack: 10 ms
- Release: 100 ms
- Makeup gain: Auto

**Algorithm:**
1. Split audio into 3 bands using Linkwitz-Riley crossovers
2. Compress each band independently
3. Apply auto makeup gain
4. Re-sum bands

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct BandCompressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup_gain: f32,
    // Compressor state
    envelope: f32,
}

impl BandCompressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 0.1,  // -20 dBFS
            ratio: 2.0,
            attack: 0.01,   // 10ms
            release: 0.1,   // 100ms
            makeup_gain: 1.0,
            envelope: 0.0,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
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

        // Auto makeup gain (simplified)
        output * self.makeup_gain
    }
}

pub struct DynamicBoostProcessor {
    low_band: BandCompressor,
    mid_band: BandCompressor,
    high_band: BandCompressor,
    // Crossover filters (4th order Linkwitz-Riley = cascade of 2nd order Butterworth)
    low_mid_lowpass: ButterLowPass<f32>,
    low_mid_highpass: ButterHighPass<f32>,
    mid_high_lowpass: ButterLowPass<f32>,
    mid_high_highpass: ButterHighPass<f32>,
    amount: f32,
    sample_rate: f32,
    crossover_low_mid: f32,
    crossover_mid_high: f32,
}

impl DynamicBoostProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let low_mid = 250.0;
        let mid_high = 4000.0;

        Self {
            low_band: BandCompressor::new(sample_rate),
            mid_band: BandCompressor::new(sample_rate),
            high_band: BandCompressor::new(sample_rate),
            low_mid_lowpass: butterlp_hz(low_mid),
            low_mid_highpass: butterhp_hz(low_mid),
            mid_high_lowpass: butterlp_hz(mid_high),
            mid_high_highpass: butterhp_hz(mid_high),
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

    pub fn process(&mut self, input: f32) -> f32 {
        if self.amount == 0.0 {
            return input;
        }

        // Split into 3 bands
        let low_mid_lp = self.low_mid_lowpass.tick(input);
        let low_mid_hp = self.low_mid_highpass.tick(input);
        let mid = self.mid_high_lowpass.tick(low_mid_hp);
        let high = self.mid_high_highpass.tick(low_mid_hp);

        // Compress each band
        let dt = 1.0 / 48000.0; // Assuming 48kHz
        let low_compressed = self.low_band.process(low_mid_lp, dt);
        let mid_compressed = self.mid_band.process(mid, dt);
        let high_compressed = self.high_band.process(high, dt);

        // Re-sum bands
        low_compressed + mid_compressed + high_compressed
    }
}
```

---

### 6. Surround Sound

**Purpose:** Stereo widening for immersive experience.

**Parameters:**
- Amount: 0.0 to 1.0 (normalized)
- Width: Controlled by amount
- Haas Delay: 2-5 ms (fixed)

**Algorithm:**
1. Mid-side encoding: Mid = (L+R)/2, Side = (L-R)/2
2. Boost side channel based on amount
3. Apply Haas delay (very short, creates wideness without phase issues)
4. Mid-side decoding: L = Mid + Side, R = Mid - Side

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct SurroundProcessor {
    side_boost: f32,
    haas_delay: Pass<f32>,  // Delay line for Haas effect
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
            side_boost: 0.0,
            haas_delay: pass(),
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
        self.side_boost = self.amount * 6.0;
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
        let delayed_side = self.delay_buffer[(self.delay_index + self.delay_buffer.len() - self.delay_samples) % self.delay_buffer.len()];
        self.delay_index = (self.delay_index + 1) % self.delay_buffer.len();

        // Boost delayed side channel
        let boost_factor = 10.0_f32.powf(self.side_boost / 20.0);
        let side_enhanced = delayed_side * boost_factor;

        // Mid-side decoding
        let output_left = mid + side_enhanced;
        let output_right = mid - side_enhanced;

        (output_left, output_right)
    }
}
```

---

### 7. Ambiance (Reverb)

**Purpose:** Simulates larger spaces for depth and immersion.

**Parameters:**
- Amount: 0.0 to 1.0 (normalized = wet/dry mix)
- Room Size: Controlled by amount
- Decay: ~1.5 to 3.0 seconds (adjustable)

**Algorithm:**
- Freeverb (Schroeder reverb topology)
- 8 parallel comb filters + 4 all-pass filters in series
- Wet/dry mixing based on amount

**Code Structure:**
```rust
use fundsp::hacker::*;

pub struct AmbianceProcessor {
    reverb: ReverbStereo<f32>,
    amount: f32,
    wet_level: f32,
    dry_level: f32,
    sample_rate: f32,
}

impl AmbianceProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            reverb: reverb_stereo(0.5),  // Default medium room
            amount: 0.0,
            wet_level: 0.0,
            dry_level: 1.0,
            sample_rate,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);

        // Map amount to reverb parameters
        // Amount 0 = no reverb, Amount 1 = full wet, large room
        let room_size = self.amount;  // 0 to 1
        self.reverb = reverb_stereo(room_size);

        // Wet/dry mixing
        // Amount 0 = 100% dry, Amount 0.5 = 50/50, Amount 1 = 100% wet
        self.wet_level = self.amount;
        self.dry_level = 1.0 - (self.amount * 0.5); // Never go to 0 dry
    }

    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.amount == 0.0 {
            return (left, right);
        }

        let (wet_left, wet_right) = self.reverb.tick((left, right));

        // Mix wet and dry
        let output_left = left * self.dry_level + wet_left * self.wet_level;
        let output_right = right * self.dry_level + wet_right * self.wet_level;

        (output_left, output_right)
    }
}
```

---

## Step-by-Step Implementation

### Step 1: Create Processor Modules

Create separate modules for each processor:

```bash
fxsonic-dsp/src/
├── lib.rs              # Main plugin interface
├── processor.rs        # Main AudioProcessor orchestrator
├── limiter.rs          # Output limiter
├── eq.rs              # Parametric EQ
├── bass_boost.rs      # Bass boost effect
├── clarity.rs         # Clarity effect
├── dynamic_boost.rs   # Multiband compressor
├── surround.rs        # Stereo widening
└── ambiance.rs        # Reverb
```

### Step 2: Implement Main AudioProcessor

Create `fxsonic-dsp/src/processor.rs`:

```rust
use std::sync::Arc;
use once_cell::sync::Lazy;

pub mod limiter;
pub mod eq;
pub mod bass_boost;
pub mod clarity;
pub mod dynamic_boost;
pub mod surround;
pub mod ambiance;

use limiter::LimiterProcessor;
use eq::EQProcessor;
use bass_boost::BassBoostProcessor;
use clarity::ClarityProcessor;
use dynamic_boost::DynamicBoostProcessor;
use surround::SurroundProcessor;
use ambiance::AmbianceProcessor;

#[derive(Clone, Copy, Debug)]
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

impl Default for AudioParams {
    fn default() -> Self {
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
            params: AudioParams::default(),
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
            let (l, r) = self.surround.process_stereo(l, r);

            // 4. Dynamic Boost
            l = self.dynamic_boost.process(l);
            r = self.dynamic_boost.process(r);

            // 5. EQ
            l = self.eq.process(l);
            r = self.eq.process(r);

            // 6. Ambiance (after EQ to keep reverb clean)
            let (l, r) = self.ambiance.process_stereo(l, r);

            left[i] = l;
            right[i] = r;
        }

        // 7. Limiter (always last)
        self.limiter.process(left, right);
    }

    pub fn get_eq_frequency_response(&self, frequencies: &[f32]) -> Vec<f32> {
        // Aggregate frequency response from all EQ bands
        self.eq.frequency_response(frequencies)
    }
}

// Global processor instance (lock-free access via Arc<AtomicRefCell>)
static PROCESSOR: Lazy<Arc<std::sync::Mutex<AudioProcessor>>> =
    Lazy::new(|| Arc::new(std::sync::Mutex::new(AudioProcessor::new(48000.0))));

pub fn get_processor() -> Arc<std::sync::Mutex<AudioProcessor>> {
    PROCESSOR.clone()
}
```

### Step 3: Integrate with LADSPA Plugin

Update `fxsonic-dsp/src/lib.rs` to use the AudioProcessor:

```rust
pub mod processor;

use ladspa::*;
use std::sync::Arc;
use processor::{AudioProcessor, AudioParams, get_processor};
use once_cell::sync::Lazy;

// Port indices
pub const PORT_BASS_BOOST: usize = 0;
pub const PORT_CLARITY: usize = 1;
pub const PORT_AMBIANCE: usize = 2;
pub const PORT_SURROUND: usize = 3;
pub const PORT_DYNAMIC_BOOST: usize = 4;
pub const PORT_ENABLED: usize = 5;
pub const PORT_EQ_1: usize = 6;
pub const PORT_EQ_2: usize = 7;
pub const PORT_EQ_3: usize = 8;
pub const PORT_EQ_4: usize = 9;
pub const PORT_EQ_5: usize = 10;
pub const PORT_EQ_6: usize = 11;
pub const PORT_EQ_7: usize = 12;
pub const PORT_EQ_8: usize = 13;
pub const PORT_EQ_9: usize = 14;
pub const PORT_EQ_10: usize = 15;
pub const PORT_AUDIO_IN_L: usize = 16;
pub const PORT_AUDIO_IN_R: usize = 17;
pub const PORT_AUDIO_OUT_L: usize = 18;
pub const PORT_AUDIO_OUT_R: usize = 19;

struct FxSonicInstance {
    // Port pointers (provided by LADSPA host)
    ports: [*mut f32; 20],

    // Local buffer for processing
    sample_rate: f32,
}

impl Plugin for FxSonicInstance {
    fn instantiate(_descriptor: &Descriptor, sample_rate: u32) -> Option<InstancePtr> {
        Box::into_raw(Box::new(FxSonicInstance {
            ports: [std::ptr::null_mut(); 20],
            sample_rate: sample_rate as f32,
        })).cast()
    }

    fn cleanup(instance: InstancePtr) {
        unsafe {
            let _ = Box::from_raw(instance.cast::<FxSonicInstance>());
        }
    }

    fn connect_port(instance: InstancePtr, port: usize, data_location: *mut f32) {
        unsafe {
            let instance = &mut *(instance.cast::<FxSonicInstance>());
            instance.ports[port] = data_location;
        }
    }

    fn activate(_instance: InstancePtr) {}

    fn deactivate(_instance: InstancePtr) {}

    fn run(instance: InstancePtr, sample_count: usize) {
        unsafe {
            let instance = &mut *(instance.cast::<FxSonicInstance>());

            // Get control port values
            let bass_boost = *instance.ports[PORT_BASS_BOOST];
            let clarity = *instance.ports[PORT_CLARITY];
            let ambiance = *instance.ports[PORT_AMBIANCE];
            let surround = *instance.ports[PORT_SURROUND];
            let dynamic_boost = *instance.ports[PORT_DYNAMIC_BOOST];
            let enabled = *instance.ports[PORT_ENABLED] > 0.5;

            let eq_gains = [
                *instance.ports[PORT_EQ_1],
                *instance.ports[PORT_EQ_2],
                *instance.ports[PORT_EQ_3],
                *instance.ports[PORT_EQ_4],
                *instance.ports[PORT_EQ_5],
                *instance.ports[PORT_EQ_6],
                *instance.ports[PORT_EQ_7],
                *instance.ports[PORT_EQ_8],
                *instance.ports[PORT_EQ_9],
                *instance.ports[PORT_EQ_10],
            ];

            // Build params struct
            let params = AudioParams {
                enabled,
                bass_boost,
                clarity,
                ambiance,
                surround,
                dynamic_boost,
                eq_gains,
            };

            // Update processor
            if let Ok(mut processor) = get_processor().lock() {
                processor.set_params(params);

                // Get audio buffers
                let input_l = std::slice::from_raw_parts_mut(instance.ports[PORT_AUDIO_IN_L], sample_count);
                let input_r = std::slice::from_raw_parts_mut(instance.ports[PORT_AUDIO_IN_R], sample_count);
                let output_l = std::slice::from_raw_parts_mut(instance.ports[PORT_AUDIO_OUT_L], sample_count);
                let output_r = std::slice::from_raw_parts_mut(instance.ports[PORT_AUDIO_OUT_R], sample_count);

                // Copy input to output (in-place processing)
                output_l.copy_from_slice(input_l);
                output_r.copy_from_slice(input_r);

                // Process audio
                processor.process_stereo(output_l, output_r);
            }
        }
    }
}

// Plugin descriptor
static DESCRIPTOR: Lazy<Descriptor> = Lazy::new(|| {
    let port_descriptions = vec![
        // Control ports
        PortDescriptor {
            name: c"Bass Boost".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Clarity".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Ambiance".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Surround Sound".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Dynamic Boost".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Enabled".as_ptr(),
            hint: PortHint::Control | PortHint::Toggle | PortHint::Default0,
            ..Default::default()
        },
        // EQ bands
        PortDescriptor {
            name: c"EQ 31 Hz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 62 Hz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 125 Hz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 250 Hz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 500 Hz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 1 kHz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 2 kHz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 4 kHz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 8 kHz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        PortDescriptor {
            name: c"EQ 16 kHz".as_ptr(),
            hint: PortHint::Control | PortHint::Default0 | PortHint::BoundedBelow0 | PortHint::BoundedAbove1,
            ..Default::default()
        },
        // Audio ports
        PortDescriptor {
            name: c"Input L".as_ptr(),
            hint: PortHint::Audio | PortHint::Input,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Input R".as_ptr(),
            hint: PortHint::Audio | PortHint::Input,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Output L".as_ptr(),
            hint: PortHint::Audio | PortHint::Output,
            ..Default::default()
        },
        PortDescriptor {
            name: c"Output R".as_ptr(),
            hint: PortHint::Audio | PortHint::Output,
            ..Default::default()
        },
    ];

    Descriptor {
        unique_id: 424242,
        label: c"fxsonic_enhancer".as_ptr(),
        properties: PluginProperties::HARD_RT_CAPABLE,
        name: c"FxSonic Audio Enhancer".as_ptr(),
        maker: c"FxSonic Project".as_ptr(),
        copyright: c"MIT".as_ptr(),
        port_count: 20,
        port_descriptions,
    }
});

#[no_mangle]
pub extern "C" fn ladspa_descriptor(index: u32) -> *const Descriptor {
    if index == 0 {
        &DESCRIPTOR as *const _
    } else {
        std::ptr::null()
    }
}
```

---

## Testing Strategy

### Unit Tests per Effect

For each effect, create comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f32, b: f32, epsilon: f32) {
        assert!((a - b).abs() < epsilon,
                "Values not close: {} vs {} (epsilon: {})", a, b, epsilon);
    }

    #[test]
    fn test_bass_boost_zero_amount() {
        let mut processor = BassBoostProcessor::new(48000.0);
        processor.set_amount(0.0);

        let input = 0.5;
        let output = processor.process(input);

        // With zero amount, should be passthrough
        assert_close(output, input, 0.001);
    }

    #[test]
    fn test_bass_boost_low_frequency() {
        let mut processor = BassBoostProcessor::new(48000.0);
        processor.set_amount(0.5);

        // Test with 100 Hz sine wave (simplified)
        // In real test, would generate actual sine wave
        let input = 0.3;
        let output = processor.process(input);

        // Output should be amplified
        assert!(output > input);
    }

    #[test]
    fn test_eq_band_response() {
        let mut processor = EQProcessor::new(48000.0);
        processor.set_band_gain(0, 6.0); // +6 dB at 31 Hz

        let frequencies = vec![31.0, 100.0, 1000.0];
        let response = processor.frequency_response(&frequencies);

        // 31 Hz should have +6 dB gain
        assert_close(response[0], 6.0, 0.5);

        // Other frequencies should have minimal gain
        assert_close(response[1], 0.0, 1.0);
        assert_close(response[2], 0.0, 1.0);
    }
}
```

### Integration Tests

Create `fxsonic-dsp/tests/integration_test.rs`:

```rust
use std::fs::File;
use std::io::BufWriter;

#[test]
fn test_full_pipeline() {
    // Generate test signal
    let sample_rate = 48000.0;
    let duration_sec = 2.0;
    let num_samples = (sample_rate * duration_sec) as usize;

    let mut left: Vec<f32> = Vec::with_capacity(num_samples);
    let mut right: Vec<f32> = Vec::with_capacity(num_samples);

    // Generate sine wave sweep
    for i in 0..num_samples {
        let t = i as f32 / sample_rate;
        let freq = 20.0 * 2.0_f32.powf(t / duration_sec * 10.0); // 20 Hz to 20 kHz
        let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3;
        left.push(sample);
        right.push(sample);
    }

    // Process with all effects enabled
    let mut processor = AudioProcessor::new(sample_rate);
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.5,
        clarity: 0.5,
        ambiance: 0.3,
        surround: 0.4,
        dynamic_boost: 0.5,
        eq_gains: [3.0, 2.0, 1.0, 0.0, -1.0, -2.0, 0.0, 2.0, 3.0, 4.0],
    });

    processor.process_stereo(&mut left, &mut right);

    // Verify no clipping (limiter should have prevented it)
    let max_sample = left.iter()
        .chain(right.iter())
        .fold(0.0f32, |acc, &x| acc.max(x.abs()));

    assert!(max_sample <= 0.8, "Output clipped! Max: {}", max_sample);

    // Write output file for manual inspection
    let output_file = BufWriter::new(
        File::create("test_output.pcm").expect("Failed to create output file")
    );

    // Write interleaved PCM
    for i in 0..num_samples {
        output_file.write_all(&left[i].to_le_bytes()).unwrap();
        output_file.write_all(&right[i].to_le_bytes()).unwrap();
    }
}
```

### Performance Benchmarks

Create `fxsonic-dsp/benches/processing_bench.rs`:

```rust
#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_processing() {
        let sample_rate = 48000.0;
        let buffer_size = 512; // Typical PipeWire buffer

        let mut processor = AudioProcessor::new(sample_rate);
        processor.set_params(AudioParams {
            enabled: true,
            bass_boost: 0.5,
            clarity: 0.5,
            ambiance: 0.3,
            surround: 0.4,
            dynamic_boost: 0.5,
            eq_gains: [0.0; 10],
        });

        // Warmup
        let mut left = vec![0.0; buffer_size];
        let mut right = vec![0.0; buffer_size];
        processor.process_stereo(&mut left, &mut right);

        // Benchmark
        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            processor.process_stereo(&mut left, &mut right);
        }

        let duration = start.elapsed();
        let total_samples = iterations * buffer_size;
        let seconds = duration.as_secs_f64();

        println!("Processed {} samples in {:.2} ms", total_samples, duration.as_millis());
        println!("Performance: {:.2}x real-time", total_samples as f64 / (sample_rate as f64 * seconds));

        // Target: at least 10x real-time (i.e., < 5ms for 512 samples at 48kHz)
        let realtime_factor = total_samples as f64 / (sample_rate as f64 * seconds);
        assert!(realtime_factor > 10.0, "Too slow: {:.2}x real-time", realtime_factor);
    }
}
```

---

## Performance Benchmarks

### Target Metrics

| Metric | Target | Method |
|--------|--------|--------|
| Processing latency | < 10 ms | 512 samples at 48kHz |
| CPU usage | < 5% (single core) | `htop` during playback |
| Memory usage | < 50 MB | `pmap` or `ps` |
| Real-time capability | > 10x real-time | Benchmark test |

### Measuring Latency

```bash
# Start PipeWire with latency monitoring
pw-top

# Observe the quantum (buffer size) and latency
# Target: < 10 ms
```

### Measuring CPU Usage

```bash
# Monitor CPU during playback
top -p $(pgrep -f fxsonic)

# Or use htop
htop -p $(pgrep -f fxsonic)
```

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin fxsonic-dsp --bench processing_bench

# View flamegraph
flamegraph.svg
```

---

## Verification Checklist

### Per Effect

- [ ] Effect at 0.0 amount = clean passthrough (within ±0.1 dB)
- [ ] Effect at 1.0 amount = audible character change
- [ ] No clicks/pops when changing amount rapidly
- [ ] No clipping (output < -0.3 dBFS)
- [ ] Stereo balance preserved (L/R difference < 0.5 dB)
- [ ] Frequency response measured and documented

### Full Pipeline

- [ ] All effects bypassed = clean passthrough
- [ ] All effects at max = no clipping (limiter working)
- [ ] Parameter changes real-time (no glitch)
- [ ] EQ curve matches visual representation
- [ ] Spectrum analyzer (if implemented) shows expected changes

### Performance

- [ ] Processing latency < 10 ms (512 samples at 48kHz)
- [ ] CPU usage < 5% during playback
- [ ] No memory leaks (stable over 1 hour)
- [ ] > 10x real-time processing speed

### PipeWire Integration

- [ ] LADSPA plugin loads without errors
- [ ] Audio flows through virtual sink
- [ ] Control ports respond to changes
- [ ] Survives PipeWire restart
- [ ] Works with various buffer sizes (64-4096 samples)

---

## Troubleshooting Guide

### Common Issues

**Issue: Audio glitches/crackles**
- Cause: Processing too slow or buffer too small
- Solution: Check CPU usage, increase PipeWire quantum, optimize DSP code

**Issue: No audio output**
- Cause: Virtual sink not connected, limiter at 0 dB
- Solution: Check PipeWire graph with `pw-dot`, verify bypass mode works

**Issue: Distortion at high settings**
- Cause: Effects creating clipping before limiter
- Solution: Adjust gain staging, reduce effect amounts

**Issue: Stereo field collapsed**
- Cause: Mid/side processing error in Surround effect
- Solution: Verify encoding/decoding: L = M+S, R = M-S

**Issue: Memory leak**
- Cause: Vectors growing in audio callback
- Solution: Ensure all buffers pre-allocated, no `Vec::push` in process()

---

## Next Steps After DSP Implementation

Once all effects are implemented and tested:

1. **Integrate with Tauri Backend**
   - Update IPC commands to control DSP parameters
   - Implement real-time parameter updates via shared memory
   - Add FFT analyzer for spectrum visualization

2. **PipeWire Filter-Chain Setup**
   - Generate `pipewire.conf.d/fxsonic-filter.conf`
   - Test with real system audio
   - Implement default sink switching

3. **Frontend Integration**
   - Connect sliders to DSP parameters
   - Implement EQ curve visualization
   - Add spectrum analyzer display

4. **Fine-Tuning**
   - Compare with FX Sound reference
   - Adjust presets for best sound quality
   - Gather user feedback

---

## Resources

### Reference Implementations

- [FxSound DSP Source](https://github.com/fxsound2/fxsound-app/tree/master/dsp) - Study filter topologies
- [FunDSP Examples](https://github.com/SamiPerttu/fundsp/tree/master/examples) - DSP patterns
- [LADSPA SDK](https://www.ladspa.org/ladspa_sdk/1.13/ladspa.h.txt) - Plugin interface

### Audio Theory

- [Audio EQ Cookbook](https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html) - Biquad formulas
- [Digital Signal Processing 101](https://www.dspguide.com/) - Comprehensive guide
- [Freeverb Paper](https://ccrma.stanford.edu/~jos/pasp/Freeverb.html) - Reverb algorithm

### Testing Tools

- [RAVEN](https://github.com/csteinmetz1/raven) - Audio analysis in Python
- [Sonic Visualiser](https://www.sonicvisualiser.org/) - Waveform/spectrum viewing
- [LADSPA Hosts](https://www.ladspa.org/cvs/ladspa_sdk/host/ladspa_h.html) - For standalone testing

---

**Document Version:** 1.0
**Last Updated:** 2026-02-25
**Next Document:** PIPEWIRE-INTEGRATION.md
