# FxSonic Testing Guide

This document provides comprehensive instructions for testing the FxSonic audio enhancement application.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Running the Application](#running-the-application)
3. [Testing the DSP Engine (CLI)](#testing-the-dsp-engine-cli)
4. [Testing the GUI Application](#testing-the-gui-application)
5. [Verifying PipeWire Integration](#verifying-pipewire-integration)
6. [Testing Audio Effects](#testing-audio-effects)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

Before testing, ensure your system meets these requirements:

**Operating System:**
- Linux with PipeWire audio stack
- Tested on: Ubuntu 22.10+, Debian 12 Bookworm+, Fedora 34+, Arch Linux

**Required System Packages:**

```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install -y \
    pipewire \
    pipewire-pulse \
    pipewire-audio-client-libraries \
    libpipewire-0.3-dev \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    imagemagick

# Fedora
sudo dnf install -y \
    pipewire \
    pipewire-pulseaudio \
    pipewire-devel \
    webkit2gtk3 \
    gtk3 \
    ImageMagick

# Arch
sudo pacman -S pipewire pipewire-pulse pipewire-jack libpipewire webkit2gtk gtk3 imagemagick
```

**Verify PipeWire is running:**

```bash
# Check if PipeWire is active
systemctl --user status pipewire pipewire-pulse wireplumber

# Check default audio devices
pactl info
```

---

## Running the Application

### Option 1: Run Pre-built Binary

The application binary is located at:
```
/home/mohiuddin/code/fx-sound/target/release/fxsonic-app
```

```bash
# Navigate to project directory
cd /home/mohiuddin/code/fx-sound

# Run the application
./target/release/fxsonic-app
```

### Option 2: Run from Source

If you want to run a debug build:

```bash
# Set required environment variable for bindgen
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include"

# Run in debug mode
cargo run --package fxsonic-app
```

### Option 3: Run via Tauri CLI

```bash
cd /home/mohiuddin/code/fx-sound/fxsonic-app

# Run with Tauri CLI (includes hot reload for frontend)
BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include" cargo tauri dev
```

---

## Testing the DSP Engine (CLI)

Before testing the GUI, verify the DSP engine works correctly using the CLI test tool.

### Run Interactive DSP Test

```bash
cd /home/mohiuddin/code/fx-sound

# Run the interactive DSP test tool
cargo run --example test_dsp
```

### Interactive Commands

Once running, you'll see a prompt. Use these commands:

| Command | Description | Example |
|---------|-------------|---------|
| `b [0-100]` | Set Bass Boost level | `b 50` |
| `c [0-100]` | Set Clarity level | `c 60` |
| `a [0-100]` | Set Ambiance level | `a 30` |
| `s [0-100]` | Set Surround Sound level | `s 40` |
| `d [0-100]` | Set Dynamic Boost level | `d 50` |
| `e` | Show EQ bands | `e` |
| `e [band] [gain]` | Set EQ band gain (±12dB) | `e 0 3` |
| `t` | Toggle power on/off | `t` |
| `r` | Reset to defaults | `r` |
| `h` | Show help | `h` |
| `q` | Quit | `q` |

### Expected Behavior

1. **Startup:** Should display "FxSonic DSP Interactive Test" with current parameters
2. **Commands:** Each command should update the displayed parameters
3. **Power toggle:** "t" command should toggle between "Power: ON" and "Power: OFF"
4. **EQ display:** "e" command shows all 10 EQ bands with frequencies and gains
5. **Quit:** "q" command should exit cleanly

### Quick Test Sequence

```
> cargo run --example test_dsp

# Test basic effects
> b 75
> c 60
> a 40
> s 50
> d 65

# Test EQ
> e 0 4        # Boost 62Hz by 4dB
> e 5 -2       # Cut 1kHz by 2dB
> e 9 6        # Boost 16kHz by 6dB

# Test power toggle
> t
> t

# Reset and exit
> r
> q
```

### Run Unit Tests

```bash
cd /home/mohiuddin/code/fx-sound

# Run all tests (should pass 63/63)
cargo test --package fxsonic-dsp

# Run tests with output
cargo test --package fxsonic-dsp -- --nocapture

# Run specific test
cargo test --package fxsonic-dsp test_bass_boost
```

---

## Testing the GUI Application

### Initial Startup

1. **Launch the application:**
   ```bash
   ./target/release/fxsonic-app
   ```

2. **Expected behavior:**
   - Window opens with title "FxSonic"
   - Dark theme UI appears (dark background, red accents)
   - Power toggle button visible
   - 5 effect sliders visible
   - EQ curve visualization displayed
   - No console errors

### GUI Component Testing

#### 1. Power Toggle Button

**Test steps:**
- Click the power button
- Visual feedback should appear (button changes state/color)
- Click again to toggle off

**Expected result:** Button toggles between ON/OFF states with smooth animation

#### 2. Effect Sliders

**Test steps for each slider:**
1. **Bass Boost slider:**
   - Move slider from 0 to 100
   - Move back to 0
   - Try positions: 25, 50, 75

2. **Clarity slider:**
   - Same as above

3. **Ambiance slider:**
   - Same as above

4. **Surround Sound slider:**
   - Same as above

5. **Dynamic Boost slider:**
   - Same as above

**Expected results:**
- Sliders move smoothly
- No lag or stuttering
- Values update in real-time
- Sliders return to previous positions when moved back

#### 3. EQ Curve Display

**Test steps:**
- Observe the EQ curve visualization
- Adjust individual EQ bands (if UI supports band selection)
- Observe curve changes

**Expected results:**
- Curve displays frequency response
- Curve updates smoothly when parameters change
- No visual artifacts

#### 4. Preset Selector

**Test steps:**
- Open preset dropdown
- Select "General"
- Select "Music"
- Select "Bass Boost"
- Select "Streaming Video"

**Expected results:**
- All presets load successfully
- Effect sliders move to preset positions
- EQ curve updates to match preset

#### 5. Device Selector

**Test steps:**
- Open device dropdown
- Observe available audio devices
- Select different output device (if multiple available)

**Expected results:**
- Lists all PipeWire audio devices
- Default device is pre-selected
- Device can be changed

### Integration Testing

#### Test with Real Audio

1. **Start playing music:**
   ```bash
   # Using mpv (example)
   mpv --audio-device=pipewire your-music-file.mp3

   # Or using vlc
   vlc your-music-file.mp3
   ```

2. **Open FxSonic GUI:**
   ```bash
   ./target/release/fxsonic-app
   ```

3. **Adjust effects while music plays:**
   - Increase Bass Boost → should hear deeper bass
   - Increase Clarity → should hear clearer highs
   - Increase Ambiance → should hear more reverb
   - Toggle Surround → should hear wider stereo

**Expected results:**
- Music plays through FxSonic
- Effect changes are audible in real-time
- No audio glitches or dropouts
- Latency should be imperceptible (< 10ms)

---

## Verifying PipeWire Integration

### Check PipeWire Nodes

```bash
# List all audio nodes
pactl list short nodes

# Look for FxSonic-related nodes
pactl list short nodes | grep -i fxsonic

# Check default sink
pactl get-default-sink
```

**Expected output:**
- Should see "FxSonic" or "fxsonic_sink" in the node list
- Default sink may or may not be FxSonic (depends on configuration)

### Monitor PipeWire Graph

```bash
# Install pw-top if not present
sudo apt-get install pipewire-tools  # Debian/Ubuntu

# Run PipeWire top
pw-top
```

**Expected behavior:**
- FxSonic node appears in the list
- Shows audio processing activity when music plays

### Test with System Audio

1. **Set FxSonic as default output:**
   ```bash
   # Find FxSonic sink name
   pactl list short sinks

   # Set as default (replace <SINK_NAME>)
   pactl set-default-sink <SINK_NAME>
   ```

2. **Play system audio:**
   - Play a YouTube video in browser
   - Use system sounds
   - Play any audio from any application

3. **Verify audio enhancement:**
   - All audio should route through FxSonic
   - Effect changes should apply to all audio

---

## Testing Audio Effects

### Bass Boost Test

**Setup:**
- Play a track with prominent bass (electronic, hip-hop)

**Test:**
1. Start with Bass Boost at 0
2. Gradually increase to 100
3. Listen for deeper, more prominent bass
4. Reduce back to 0 to compare

**Expected:**
- Bass becomes stronger and deeper
- Low-end frequency response improves
- No distortion or muddiness at moderate levels

### Clarity Test

**Setup:**
- Play vocals or acoustic track

**Test:**
1. Start with Clarity at 0
2. Increase to 50-60
3. Listen for clearer, more articulate vocals
4. Toggle on/off to compare

**Expected:**
- Vocals become more intelligible
- High-frequency detail increases
- Sibilance controlled (not harsh)

### Ambiance Test

**Setup:**
- Play dry, close-mic recording

**Test:**
1. Start with Ambiance at 0
2. Increase to 30-50
3. Listen for added space/reverb
4. Compare on/off

**Expected:**
- Sound appears to be in a larger room
- Wet/dry mix is smooth
- No metallic or artificial artifacts

### Surround Sound Test

**Setup:**
- Play stereo recording with wide panning

**Test:**
1. Start with Surround at 0
2. Increase to 40-60
3. Listen for wider stereo image
4. Compare on/off

**Expected:**
- Stereo width increases
- Instruments feel more spacious
- Center image remains stable

### Dynamic Boost Test

**Setup:**
- Play dynamic track (quiet parts, loud parts)

**Test:**
1. Start with Dynamic Boost at 0
2. Increase to 50-70
3. Listen for compressed/consistent volume
4. Compare on/off

**Expected:**
- Quieter parts become louder
- Loud parts don't clip
- Overall volume more consistent

### EQ Test

**Setup:**
- Play full-range music track

**Test:**
1. Set all EQ bands to 0 (flat)
2. Boost low bands (62Hz, 125Hz) → should hear more bass
3. Boost high bands (8kHz, 16kHz) → should hear more air/crispness
4. Cut mid band (1kHz) → should hear less mids
5. Reset to flat

**Expected:**
- Each band affects its frequency range
- EQ curve matches visual display
- Changes are smooth and musical

---

## Troubleshooting

### Application Won't Start

**Symptom:** Binary runs but window doesn't appear

**Solutions:**
1. Check if WebKitGTK is installed:
   ```bash
   dpkg -l | grep webkit2gtk
   ```

2. Verify PipeWire is running:
   ```bash
   systemctl --user status pipewire
   ```

3. Check for missing libraries:
   ```bash
   ldd ./target/release/fxsonic-app | grep "not found"
   ```

### No Audio Enhancement

**Symptom:** Application runs but audio sounds unchanged

**Solutions:**
1. Check if default sink is FxSonic:
   ```bash
   pactl get-default-sink
   ```

2. Verify FxSonic node exists:
   ```bash
   pactl list short sinks | grep -i fxsonic
   ```

3. Restart PipeWire:
   ```bash
   systemctl --user restart pipewire pipewire-pulse
   ```

4. Check application logs:
   ```bash
   ./target/release/fxsonic-app 2>&1 | tee fxsonic.log
   ```

### Audio Glitches or Dropouts

**Symptom:** Popping, crackling, or audio cutting out

**Solutions:**
1. Check PipeWire buffer size:
   ```bash
   pw-top
   ```

2. Increase PipeWire quantum (lower priority):
   Edit `~/.config/pipewire/pipewire.conf.d/99-custom.conf`:
   ```
   context.properties = {
       default.clock.quantum = 1024
   }
   ```

3. Close other audio applications to reduce CPU load

4. Check CPU usage:
   ```bash
   top -p $(pgrep fxsonic-app)
   ```

### GUI Not Responsive

**Symptom:** Sliders lag, window freezes

**Solutions:**
1. Check memory usage:
   ```bash
   ps aux | grep fxsonic-app
   ```

2. Restart application

3. Check for JavaScript errors (if dev mode):
   ```bash
   BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include" cargo tauri dev
   ```

### Build Errors

**Symptom:** `cargo build` or `cargo tauri build` fails

**Common errors:**

1. **Missing stdbool.h:**
   ```bash
   export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include"
   ```

2. **Missing GTK headers:**
   ```bash
   sudo apt-get install libgtk-3-dev libglib2.0-dev libwebkit2gtk-4.1-dev
   ```

3. **Missing PipeWire headers:**
   ```bash
   sudo apt-get install libpipewire-0.3-dev
   ```

4. **OUT_DIR error:**
   - Ensure `build.rs` exists in `fxsonic-app/` directory
   - Use `cargo tauri build` instead of `cargo build`

### PipeWire Integration Issues

**Symptom:** "Operation not supported" or similar errors

**Solutions:**
1. Verify PipeWire version:
   ```bash
   pipwire --version
   ```
   Should be version 0.3.60 or later

2. Check permissions:
   ```bash
   groups $USER
   ```
   Should include `audio` group

3. Check filter-chain module:
   ```bash
   pactl list modules | grep filter-chain
   ```

---

## Performance Benchmarks

### Expected Performance Metrics

| Metric | Expected Value | Test Command |
|--------|----------------|--------------|
| **CPU Usage** | < 5% (idle), < 15% (processing) | `htop` |
| **Memory Usage** | < 100MB | `ps aux | grep fxsonic` |
| **Latency** | < 10ms @ 48kHz | `pw-top` |
| **Startup Time** | < 3 seconds | `time ./target/release/fxsonic-app` |
| **DSP Processing** | < 1ms per buffer | Built-in benchmark |

### Running Benchmarks

```bash
# DSP engine benchmark
cargo test --package fxsonic-dsp --release -- --nocapture

# Profile application
cargo flamegraph --bin fxsonic-app

# Memory profiling
valgrind --leak-check=full ./target/release/fxsonic-app
```

---

## Test Checklist

Use this checklist to verify all functionality:

- [ ] Application binary exists and is executable
- [ ] Application launches without errors
- [ ] GUI window opens with dark theme
- [ ] Power toggle button works
- [ ] All 5 effect sliders respond to input
- [ ] EQ curve visualization displays
- [ ] Preset selector loads all presets
- [ ] Device selector shows audio devices
- [ ] CLI DSP test tool runs successfully
- [ ] All unit tests pass (63/63)
- [ ] Audio enhancement is audible with real music
- [ ] PipeWire nodes are created correctly
- [ ] No audio glitches or dropouts during playback
- [ ] CPU usage remains reasonable (< 15%)
- [ ] Application responds to slider changes in real-time
- [ ] Settings persist across restarts

---

## Reporting Issues

When reporting bugs, include:

1. **System Information:**
   ```bash
   uname -a
   pipewire --version
   ```

2. **Application Version:**
   ```bash
   ./target/release/fxsonic-app --version
   ```

3. **Logs:**
   ```bash
   ./target/release/fxsonic-app 2>&1 | tee bug-report.log
   ```

4. **Steps to Reproduce:**
   - Clear description of what you were doing
   - Expected vs actual behavior
   - Screenshots if applicable

5. **Audio Test Case:**
   - Type of audio (music, video, etc.)
   - Effects being used
   - Settings that cause the issue

---

## Additional Resources

- **Project README:** `/home/mohiuddin/code/fx-sound/README.md`
- **DSP Documentation:** `/home/mohiuddin/code/fx-sound/DSP-ENGINE-IMPLEMENTATION.md`
- **System Requirements:** `/home/mohiuddin/code/fx-sound/FxSonic-System-Requirements.md`
- **Build Status:** `/home/mohiuddin/code/fx-sound/BUILD-STATUS.md`

---

## Quick Start Summary

```bash
# 1. Navigate to project
cd /home/mohiuddin/code/fx-sound

# 2. Test DSP engine (CLI)
cargo run --example test_dsp

# 3. Run GUI application
./target/release/fxsonic-app

# 4. Play music to test effects
# Open your music player and adjust sliders in FxSonic

# 5. Verify PipeWire integration
pactl list short nodes | grep -i fxsonic
```

Happy testing! 🎵
