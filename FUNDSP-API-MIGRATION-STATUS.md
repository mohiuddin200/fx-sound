# Fundsp API Migration Status

**Date:** 2026-02-25
**Issue:** Fundsp 0.18 has significantly changed API from earlier versions

---

## Current Problems

### 1. Frame-Based API

Fundsp 0.18 now uses `Frame<T, N>` types (alias for `NumericArray<T, N>`) everywhere:
- `tick()` method takes `&Frame<f32, Inputs>` not `f32`
- `tick()` method returns `Frame<f32, Outputs>` not `f32`
- This applies to all filter and processor types

### 2. Type Wrappers

All filter types now need `An<>` wrapper:
- `Bell<f32>` → Not available, filters return `An<FixedSvf<f64, Mode>>`
- `LowShelf<f32>` → Not available, returns `An<FixedSvf<f64, LowshelfMode>>`
- `HighShelf<f32>` → Not available, returns `An<FixedSvf<f64, HighshelfMode>>`
- `Waveshaper` → Not available, use `tanh()` function directly
- `Follower` → Renamed to `Follow`

### 3. Internal Data Types

While fundsp's `hacker32` module uses f32 API, underlying types still use f64:
- `FixedSvf<f64, Mode>` - internal state is f64
- `Follow<f64>` - internal state is f64
- This means conversions may be needed

### 4. Struct Field Type Issue

Cannot use `An<impl AudioNode<...>>` as struct field type (not allowed in Rust).

---

## Files Requiring Fixes

### 1. eq.rs ❌
**Status:** Fixed type declarations, but `tick()` API mismatch remains

**Issues:**
- `Bell<f32>` type doesn't exist
- `tick()` expects `&Frame<f32, U1>` not `f32`
- Returns `Frame<f32, U1>` not `f32`

**Needs:** Complete rewrite using Frame API

### 2. bass_boost.rs ❌
**Status:** Fixed type declarations, but `tick()` API mismatch remains

**Issues:**
- `LowShelf<f32>` type doesn't exist
- `Waveshaper<f32>` type doesn't exist
- `tick()` expects `&Frame`
- Arithmetic with `Frame` types

**Needs:** Complete rewrite using Frame API

### 3. clarity.rs ❌
**Status:** Fixed type declarations, but `tick()` API mismatch remains

**Issues:**
- `HighShelf<f32>` type doesn't exist
- `Follower<f32>` type doesn't exist
- `follow()` now takes 1 arg
- `tick()` expects `&Frame`

**Needs:** Complete rewrite using Frame API

### 4. dynamic_boost.rs ❌
**Status:** Fixed filter types, but `tick()` API mismatch remains

**Issues:**
- `ButterLowPass<f32>` and `ButterHighPass<f32>` don't exist
- `butterlp_hz()` and `butterhp_hz()` don't exist
- Using `lowpass_hz()` and `highpass_hz()` instead
- `tick()` expects `&Frame`

**Needs:** Complete rewrite using Frame API

### 5. ambiance.rs ❌
**Status:** Partially fixed, but struct field type invalid

**Issues:**
- `ReverbStereo<f32>` type doesn't exist
- `reverb_stereo()` takes 3 args (room_size, time, damping)
- `An<impl AudioNode...>` not allowed as field type

**Needs:** Rewrite using concrete type or different storage pattern

### 6. processor.rs ✅
**Status:** Fixed mutable variable bindings

**Issues:** None (fixed)

---

## Required API Changes

### Frame Conversion

To process single sample with fundsp filters:

```rust
// Old API (doesn't exist anymore)
let output = filter.tick(input);

// New API (needs Frame)
let input_frame: Frame<f32, U1> = [input].into();
let output_frame = filter.tick(&input_frame);
let output: f32 = output_frame[0];
```

### Filter Creation

```rust
// Old (Bell filter)
let filter: Bell<f32> = bell_hz(freq, gain);

// New
let filter: An<FixedSvf<f64, BellMode<f64>>> = bell_hz(freq, q, gain);
```

### Arithmetic with Frames

```rust
// Old
let result = value1 + value2;

// New (if working with Frames)
let result_frame = frame1 + frame2;
let result = result_frame[0];
```

---

## Solution Options

### Option A: Complete Rewrite (Recommended) ⭐
Rewrite all processors to use fundsp's Frame API properly.

**Pros:**
- Correct usage of fundsp API
- Future-proof
- Maintains fundsp's optimizations

**Cons:**
- Significant code changes (~1000+ lines)
- Learning curve for Frame API
- May affect performance due to Frame allocations

**Estimated Time:** 4-6 hours

### Option B: Alternative DSP Library
Replace fundsp with a simpler, f32-focused library.

**Options:**
- `biquad-rs` - Good for filters, limited for reverb/compressor
- `dasp` - Fundamentals, may lack advanced features
- `rust-dsp` - Older, less maintained
- Raw biquad implementation - Most control, most work

**Pros:**
- Simpler API
- Direct f32 processing
- Less overhead

**Cons:**
- Need to implement more from scratch (reverb, multiband compressor)
- Less tested

**Estimated Time:** 6-8 hours

### Option C: Hybrid Approach
Use simpler library for basic filters, fundsp for complex effects (reverb).

**Pros:**
- Best of both worlds
- Can use `biquad-rs` for filters (direct f32)
- Use fundsp for reverb only

**Cons:**
- Mixed dependencies
- More complex build

**Estimated Time:** 3-4 hours

### Option D: Downgrade Fundsp
Use older fundsp version that matches original API.

**Pros:**
- Minimal code changes
- Tested API

**Cons:**
- Old version may have bugs
- Missing newer features
- Potential security vulnerabilities

**Estimated Time:** 30 minutes (to find compatible version and update Cargo.toml)

---

## Recommendation

**Option D (Downgrade) followed by Option A (Complete Rewrite):**

1. **Immediate:** Try downgrading to fundsp 0.15 or 0.16 to see if API matches
2. **If downgrade works:** Build and test, then upgrade gradually
3. **If downgrade doesn't work:** Proceed with Option A (Complete Rewrite) or Option C (Hybrid)

---

## Next Actions

1. Test fundsp 0.15.x or 0.16.x API compatibility
2. If compatible, update Cargo.toml and rebuild
3. If not compatible, decide between Option A, B, or C
4. Implement chosen solution
5. Update all processor modules
6. Run tests

---

## Progress Tracking

- [x] Identified API changes
- [x] Updated NEXT-STEPS.md with detailed issues
- [ ] Test fundsp downgrade (0.15/0.16)
- [ ] Choose implementation approach
- [ ] Rewrite/update all processor modules
- [ ] Run cargo build
- [ ] Run cargo test
- [ ] Verify audio quality
