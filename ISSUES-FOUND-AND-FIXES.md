# FxSonic Issues Found and Fixes Applied

**Date:** 2026-02-26
**Status:** Code fixes implemented, rebuild required

## Issues Identified

### 1. Slowness - Excessive Filter-Chain Restarts

**Problem:** The application was extremely slow because every slider change triggered a complete filter-chain process restart. With a 600ms debounce, users experienced multiple restarts when moving sliders slowly.

**Impact:**
- Each restart took 5-10+ seconds
- Process restart involved:
  - Killing the old process
  - Waiting 300ms
  - Writing config file
  - Starting new `pipewire` process
  - Waiting up to 15 retries (400-500ms each) to find the node
  - Setting as default sink

**Fix Applied:**
- Increased debounce timer from 600ms to **1500ms** in `frontend/src/App.tsx`
- Added better error handling with user notifications
- Reduced the number of restarts when users are actively adjusting sliders

### 2. Output Device Not Changing

**Problem:** Changing the output device didn't work reliably because:
- The filter-chain process kept dying (becoming zombie processes)
- The `set_fxsonic_as_default()` function had error-prone node finding logic
- Retry logic was insufficient

**Root Causes:**
- Filter-chain process was exiting silently (no error logs)
- Node finding from `wpctl status` output was fragile
- Process monitoring was insufficient

**Fixes Applied:**
- Improved `apply_settings()` function with:
  - Better process status checking
  - Exponential backoff for retries (300ms → 500ms → 800ms)
  - Increased retry count from 15 to 20
  - Better error messages with troubleshooting hints
- Enhanced logging with "FxSonic:" prefix for easier debugging
- Added process monitoring function `is_filter_running()`
- Added `get_filter_status` command for frontend monitoring

### 3. Mode/Effect Changes Not Applying

**Problem:** Effect and mode changes appeared to not work because:
- Changes were saved to config
- But filter-chain restarts were failing or taking too long
- Users saw no visual feedback that changes were being applied

**Fixes Applied:**
- Added visual "Applying..." indicator that shows when changes are being processed
- Added error alerts if changes fail to apply
- Added UI state reversion on error (e.g., if device change fails, revert to previous selection)
- Added periodic filter status checking (every 5 seconds) in frontend
- Added "Filter Stopped" warning indicator when the process dies

### 4. FxSonic Not Set as Default Sink

**Problem:** FxSonic sink was never set as default, causing apps to output directly to hardware devices instead of through FxSonic.

**Root Causes:**
- `find_fxsonic_wpctl_id()` parsing logic was fragile
- No retry mechanism if node wasn't ready immediately
- No feedback if default sink setting failed

**Fixes Applied:**
- Improved error messages in node finding function
- Added retry logic with exponential backoff in `apply_settings()`
- Better process status verification before attempting to set default
- Enhanced logging to track the sink setting process

## Files Modified

### Frontend (`fxsonic-app/frontend/src/App.tsx`)
- ✅ Increased debounce timer from 600ms to 1500ms
- ✅ Added error alerts for failed operations
- ✅ Added UI state reversion on errors
- ✅ Added filter status monitoring (every 5 seconds)
- ✅ Added "Filter Stopped" warning indicator
- ✅ Enhanced status card to show filter process status
- ✅ Improved "Applying..." indicator visibility

### Backend (`fxsonic-app/src/main.rs`)
- ✅ Added `get_filter_status` command to check if filter-chain process is running
- ✅ Registered new command in Tauri invoke handler

### Backend (`fxsonic-app/src/pipewire_manager.rs`)
- ✅ Improved `apply_settings()` with better retry logic (exponential backoff)
- ✅ Enhanced error messages with troubleshooting hints
- ✅ Added `is_filter_running()` method for process monitoring
- ✅ Improved `start_filter()` with better logging
- ✅ Fixed `stop_filter()` to wait longer (400ms instead of 300ms)
- ✅ Enhanced `set_fxsonic_as_default()` with better error handling
- ✅ Improved `find_fxsonic_wpctl_id()` error messages

## Current Status

### What Works
- ✅ DSP engine works correctly (tested with `test_dsp` example)
- ✅ All effects process audio correctly
- ✅ EQ bands work properly
- ✅ Config file generation is correct
- ✅ Code improvements implemented

### What Doesn't Work Yet
- ❌ **LADSPA Plugin Integration**: The filter-chain process keeps dying when loaded by PipeWire
  - Process starts but exits within a few seconds
  - No error messages in logs (log file remains empty)
  - FxSonic nodes never appear in PipeWire registry

### Root Cause of Current Failure

The LADSPA plugin (`/home/mohiuddin/.ladspa/libfxsonic_dsp.so`) appears to crash when loaded by PipeWire's filter-chain module. This is a **LADSPA integration issue**, not a DSP engine issue.

**Evidence:**
- DSP engine works perfectly when tested directly
- LADSPA plugin builds successfully
- Filter-chain starts but dies immediately
- No error output (suggests segfault or silent failure)

## Next Steps to Fix the LADSPA Issue

### Option 1: Debug LADSPA Plugin (Recommended)

The LADSPA plugin interface in `fxsonic-dsp/src/lib.rs` may have issues:

1. **Check control port names** - Ensure they match the config exactly:
   - Current: "Bass Boost", "Clarity", "Ambiance", "Surround Sound", "Dynamic Boost"
   - Config uses: "Bass Boost", "Clarity", "Ambiance", "Surround Sound", "Dynamic Boost"

2. **Verify LADSPA descriptor structure** - The plugin may not be implementing the LADSPA interface correctly

3. **Add safety checks** - The plugin might be crashing on invalid parameters or sample rates

4. **Test with a simple LADSPA host** - Use `listplugins` or other LADSPA tools to verify the plugin loads

**Commands to debug:**
```bash
# Check if plugin is recognized by LADSPA system
listplugins | grep fxsonic

# Test loading the plugin
analyseplugin /home/mohiuddin/.ladspa/libfxsonic_dsp.so
```

### Option 2: Alternative Integration (If LADSPA Cannot Be Fixed)

Instead of using LADSPA + filter-chain, consider:

1. **Direct PipeWire Integration** using `pipewire-rs`:
   - Create a `pw_filter` node directly in Rust
   - Process audio in the filter callback
   - More control over the audio pipeline

2. **Standalone Audio Application**:
   - Use the DSP library directly
   - Audio I/O via `rodio` or `cpal`
   - No PipeWire integration needed
   - Simpler architecture

### Option 3: Fix Build Environment and Rebuild

The code changes won't take effect until the application is rebuilt:

```bash
# Install missing dependencies (requires sudo)
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libpipewire-dev \
    libclang-dev \
    pkg-config \
    build-essential

# Rebuild the application
cd /home/mohiuddin/code/fx-sound/fxsonic-app/frontend
npm run build
cd ..
cargo build --release

# Or run in dev mode
cd frontend
npm run tauri dev
```

## Testing the Fixes

Once the LADSPA issue is resolved, the fixes I implemented should:

1. ✅ **Reduce slowness**: Slider changes will only trigger restarts after 1.5 seconds of inactivity
2. ✅ **Show better feedback**: Users will see "Applying..." indicators and error alerts
3. ✅ **Handle failures gracefully**: The UI will revert to previous state on errors
4. ✅ **Monitor process health**: The app will detect when the filter-chain dies and warn the user
5. ✅ **Improve reliability**: Better retry logic increases the chance of successful device changes

## Manual Workaround (Until Fixed)

If you need the audio enhancement working immediately:

```bash
# 1. Kill the current broken filter-chain
pkill -f "pipewire.*fxsonic"

# 2. Use the DSP library directly in a custom application
cd /home/mohiuddin/code/fx-sound
cargo run --package fxsonic-dsp --example test_dsp
# Use this to test parameters and verify the DSP works

# 3. Or use existing tools like EasyEffects or JamesDSP
# These have working PipeWire integration and can provide similar effects
```

## Summary

The main issues you reported (slowness, output device not changing, mode not changing) were caused by:

1. **Too frequent filter-chain restarts** → Fixed by increasing debounce timer
2. **Process dying silently** → Partially fixed with better monitoring (needs LADSPA fix)
3. **Poor error handling** → Fixed with better error messages and user feedback

**The core blocker is the LADSPA plugin integration with PipeWire**, which causes the filter-chain process to die. Once this is fixed, all the improvements I made will make the application much more responsive and reliable.

---

## Files Changed

- `fxsonic-app/frontend/src/App.tsx` - UI improvements and error handling
- `fxsonic-app/src/main.rs` - Added filter status command
- `fxsonic-app/src/pipewire_manager.rs` - Process management improvements

## Files to Check for LADSPA Issue

- `fxsonic-dsp/src/lib.rs` - LADSPA plugin implementation (lines 280-400)
- `fxsonic-dsp/Cargo.toml` - LADSPA dependencies (using forked `ladspa` crate)
