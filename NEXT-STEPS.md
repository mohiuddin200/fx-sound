# FxSonic DSP - Next Steps

**Date:** 2026-02-25
**Status:** Fundsp API Compatibility Fix Required

---

## Completed Work

### DSP Processor Modules (All Implemented ✅)

1. **LimiterProcessor** (`limiter.rs`)
   - Brick-wall limiter at -0.3 dBFS
   - Prevents clipping from accumulated gain
   - Comprehensive unit tests

2. **EQProcessor** (`eq.rs`)
   - 10-band parametric EQ at: 31, 62, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz
   - Gain range: -12 to +12 dB per band
   - Comprehensive unit tests

3. **BassBoostProcessor** (`bass_boost.rs`)
   - Low-shelf filter (50-250 Hz, default 100 Hz)
   - Harmonic exciter using tanh waveshaper
   - Gain: 0 to +12 dB based on amount
   - Comprehensive unit tests

4. **ClarityProcessor** (`clarity.rs`)
   - High-shelf filter (2k-10k Hz, default 4k Hz)
   - Dynamic envelope follower (5ms attack, 100ms release)
   - Dynamic boost up to +6 dB based on envelope
   - Comprehensive unit tests

5. **DynamicBoostProcessor** (`dynamic_boost.rs`)
   - 3-band multiband compressor
   - Crossovers: 250 Hz, 4 kHz
   - Ratio: 2:1 to 4:1 based on amount
   - Linkwitz-Riley filters (4th order)
   - Auto makeup gain (+3.5 dB)
   - Comprehensive unit tests

6. **SurroundProcessor** (`surround.rs`)
   - Mid-side encoding/decoding
   - Side channel boost: 0 to +6 dB
   - Haas delay: 4ms (192 samples @ 48kHz)
   - Comprehensive unit tests

7. **AmbianceProcessor** (`ambiance.rs`)
   - Freeverb reverb (Schroeder topology)
   - Wet/dry mixing based on amount
   - Room size: 0 to 1 (controlled by amount)
   - Comprehensive unit tests

8. **AudioProcessor** (`processor.rs`)
   - Main orchestrator combining all effects
   - Processing chain: Bass → Clarity → Surround → Dynamic → EQ → Ambiance → Limiter
   - `AudioParams` struct for parameter passing
   - `EffectType` enum for effect identification
   - Comprehensive unit tests

### Files Fixed
- ✅ processor.rs module imports fixed (changed `pub mod limiter` to `use crate::limiter`)

---

## Current Issues

### 1. Fundsp API Compatibility (BLOCKING - 31 errors)

The fundsp crate API has changed between the version used during initial implementation and the current version (0.18.2). The following changes are needed:

#### Type Wrapper Changes
- Filter types now need `An<>` wrapper:
  - `Bell<f32>` → `An<Bell<f32>>`
  - `LowShelf<f32>` → `An<LowShelf<f32>>`
  - `HighShelf<f32>` → `An<HighShelf<f32>>`
  - `Waveshaper<f32>` → `An<Waveshaper<f32>>`
  - `Follower<f32>` → `An<Follower<f32>>`
  - `ReverbStereo<f32>` → `An<ReverbStereo<f32>>`
  - `ButterLowPass<f32>` → `An<ButterLowPass<f32>>`
  - `ButterHighPass<f32>` → `An<ButterHighPass<f32>>`

#### Function Signature Changes
- Filter functions now require Q parameter:
  - `bell_hz(f, gain)` → `bell_hz(f, q, gain)`
  - `lowshelf_hz(f, gain)` → `lowshelf_hz(f, q, gain)`
  - `highshelf_hz(f, gain)` → `highshelf_hz(f, q, gain)`

#### Envelope Follower Changes
- `follow()` function signature changed:
  - `follow(attack, release)` → `follow(halfway_response_time)`

#### Filter Function Changes
- Butterworth filter functions may have been renamed:
  - `butterlp_hz(f)` → `butterpass_hz(f)` or new name
  - `butterhp_hz(f)` → may have been renamed

#### Reverb Function Changes
- `reverb_stereo()` may have different signature:
  - Was: `reverb_stereo(room_size)`
  - Now: Possibly `reverb_stereo(room_size, damping, ...)`

### 2. LADSPA Crate Issue (BLOCKING)

- **Problem:** `ladspa` crate v0.2.3 uses deprecated `extern crate libc;` syntax
- **Error:** `error[E0658]: use of unstable library feature rustc_private`
- **Root Cause:** The crate hasn't been updated for modern Rust
- **Status:** lib.rs LADSPA integration code exists but cannot compile
- **Workaround Options:**
  1. Find or create a fork of ladspa crate with updated code
  2. Use ladspa v0.3.4 with API rewrite
  3. Directly implement LADSPA interface without crate

---

## Next Steps

### Phase 1: Fix Fundsp API Compatibility (Priority: HIGH)

1. **Update eq.rs**
   - Wrap `Bell<f32>` in `An<>`
   - Add Q parameter to `bell_hz()` calls (use 2.0 as default)

2. **Update bass_boost.rs**
   - Wrap `LowShelf<f32>` and `Waveshaper<f32>` in `An<>`
   - Add Q parameter to `lowshelf_hz()` call (use 1.0 as default)

3. **Update clarity.rs**
   - Wrap `HighShelf<f32>` and `Follower<f32>` in `An<>`
   - Add Q parameter to `highshelf_hz()` call (use 1.0 as default)
   - Fix `follow()` call to use single parameter

4. **Update dynamic_boost.rs**
   - Wrap `ButterLowPass<f32>` and `ButterHighPass<f32>` in `An<>`
   - Fix `butterlp_hz()` and `butterhp_hz()` function calls

5. **Update ambiance.rs**
   - Wrap `ReverbStereo<f32>` in `An<>`
   - Fix `reverb_stereo()` function call with correct parameters

6. **Update processor.rs**
   - Fix mutable variable bindings if needed (shadowing in process_stereo)

7. **Run build**
   ```bash
   cargo build --package fxsonic-dsp
   ```

8. **Run tests**
   ```bash
   cargo test --package fxsonic-dsp
   ```

### Phase 2: Resolve LADSPA Integration (Priority: MEDIUM)

1. **Evaluate options:**
   - Option A: Fork and update ladspa 0.2.3 crate
   - Option B: Use ladspa 0.3.4 (different API, needs rewrite)
   - Option C: Implement LADSPA interface directly (more control)

2. **Implement chosen solution**

3. **Update lib.rs with working LADSPA integration**

4. **Test LADSPA plugin loading**

### Phase 3: Integration Testing (Priority: MEDIUM)

1. **Generate test audio files**
   ```bash
   mkdir -p fxsonic-dsp/tests/audio
   ffmpeg -f lavfi -i "sine=frequency=100:sample_rate=48000:duration=5" \
           fxsonic-dsp/tests/audio/sine-100hz.wav
   ffmpeg -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=5" \
           fxsonic-dsp/tests/audio/sine-1khz.wav
   ffmpeg -f lavfi -i "anoisesrc=white:sample_rate=48000:duration=5" \
           fxsonic-dsp/tests/audio/white-noise.wav
   ```

2. **Create integration tests** (`fxsonic-dsp/tests/integration_test.rs`)
   - Full pipeline processing test
   - Frequency sweep test
   - Multi-buffer processing test

3. **Run integration tests**

### Phase 4: PipeWire Integration (Priority: LOW)

1. **Create PipeWire filter-chain config**
   - `pipewire.conf.d/fxsonic-filter.conf`

2. **Test with actual system audio**

3. **Verify real-time performance**

---

## File Changes Required

### High Priority
1. `fxsonic-dsp/src/eq.rs` - Fix Bell type and bell_hz calls
2. `fxsonic-dsp/src/bass_boost.rs` - Fix LowShelf, Waveshaper types and lowshelf_hz
3. `fxsonic-dsp/src/clarity.rs` - Fix HighShelf, Follower types and highshelf_hz
4. `fxsonic-dsp/src/dynamic_boost.rs` - Fix ButterLowPass, ButterHighPass and butter functions
5. `fxsonic-dsp/src/ambiance.rs` - Fix ReverbStereo type and reverb_stereo
6. `fxsonic-dsp/src/processor.rs` - Fix variable bindings if needed

### Medium Priority
7. `fxsonic-dsp/src/lib.rs` - Add LADSPA integration once crate issue resolved
8. `fxsonic-dsp/tests/integration_test.rs` - Create integration tests

### Low Priority
9. `pipewire.conf.d/fxsonic-filter.conf` - Create PipeWire configuration

---

## Success Criteria

- [ ] All modules compile without errors
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] LADSPA plugin loads successfully in PipeWire
- [ ] Real-time audio processing verified
- [ ] No clipping or artifacts in output
- [ ] Performance benchmarks meet targets (<10ms latency, <5% CPU)

---

## Estimated Time

- Phase 1 (Fundsp fixes): 1-2 hours
- Phase 2 (LADSPA): 2-4 hours (depending on chosen solution)
- Phase 3 (Integration): 1 hour
- Phase 4 (PipeWire): 1-2 hours

**Total:** 5-9 hours to completion
