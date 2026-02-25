# FxSonic — FX Sound for Linux
## System Requirements & Technology Plan

---

## 1. Context & Problem Statement

**FX Sound** (formerly DFX Audio Enhancer) is a beloved Windows audio enhancer known for its "5 sliders + power button" simplicity and beautiful dark UI with real-time audio visualization. It has over 50 million downloads since 1999 and was recently made open-source (AGPL-3.0).

**The problem:** Linux has no equivalent. Existing open-source alternatives expose dozens of knobs, require understanding of audio signal chains, and present UIs designed for audio engineers — not casual users. Common complaints include:

- Complex, intimidating interfaces
- Steep learning curves requiring audio backend knowledge
- Poor visual design and lack of polish
- No unified "just works" experience
- Installation and configuration friction

**The goal:** Build a Linux-native audio enhancement application that matches FX Sound's simplicity, visual polish, and audio quality — built from the ground up for the modern Linux audio stack (PipeWire).

**Design philosophy:** Five main effect sliders, a power button, and better sound. Zero configuration required.

---

## 2. FX Sound Feature Reference

FX Sound ([fxsound2/fxsound-app](https://github.com/fxsound2/fxsound-app) on GitHub, built with JUCE 6.1.6) provides:

### Core Audio Processing
- **Bass Boost** — Low-shelf filtering + harmonic excitation for thump, rumble, and punch
- **Clarity** — Dynamic high-frequency enhancement for crisp, airy, articulate treble
- **Ambiance** — Algorithmic reverb simulating larger spaces (concert halls, rooms)
- **Surround Sound** — Stereo widening via mid-side processing for expansive soundscape
- **Dynamic Boost** — Multiband compression restoring amplitude lost in compression

### Equalizer
- 9-10 band parametric EQ with adjustable center frequencies
- Frequency range: 62 Hz (sub-bass) to 13,000 Hz (high treble)
- Each band has adjustable center frequency, gain, and Q factor

### UI/UX
- Dark theme with red accents (`#1a1a2e` background, `#e94560` accent)
- Real-time animated audio visualizer synced to playback
- Clean, modern sliders and circular controls
- Mini-mode / condensed view option
- System tray integration
- Customizable keyboard shortcuts for preset switching

### Presets
- General, Music, Voice, Streaming Video, Bass Boost, Light Processing
- Genre-specific: Classic Rock, Modern Rock, Pop, Modern Country, Trap, 70's, 80's

### System Integration
- Virtual audio device ("FxSound Speakers") as default playback device
- System-wide processing across all applications automatically
- "Set it and forget it" — no per-app configuration needed
- Real-time processing with no perceptible latency

---

## 3. Existing Open-Source Linux Alternatives

### Tier 1: Primary System-Wide Audio Enhancers

#### EasyEffects (formerly PulseEffects)
- **URL:** [github.com/wwmm/easyeffects](https://github.com/wwmm/easyeffects)
- **Tech:** C++, Qt/Kirigami (v8.0+), PipeWire-only
- **Stars:** ~8,700
- **Status:** Actively maintained (v8.1.2, Feb 2026)          
- **Features:** Limiter, compressor, equalizer, bass enhancer, crystalizer, de-esser, reverb, stereo tools, per-app effects, 30+ effect types
- **Limitations:** Complex UI with steep learning curve, PipeWire-only (no PulseAudio for v6.0+), overwhelming number of options for casual users

#### JamesDSP for Linux
- **URL:** [github.com/Audio4Linux/JDSP4Linux](https://github.com/Audio4Linux/JDSP4Linux)
- **Tech:** C++, Qt GUI, PipeWire + PulseAudio support
- **Status:** Active development
- **Features:** Auto bass boost, dynamic range processor, parametric/fixed-band EQ, convolver, crossfeed, stereo widening, ViPER-DDC, EEL2 scripting
- **Advantages:** Works with both PipeWire AND PulseAudio, simpler than EasyEffects, lower CPU overhead, custom scripting
- **Limitations:** Output-only, smaller community, still more complex than FX Sound

### Tier 2: PulseAudio-Specific Solutions

#### PulseAudio Equalizer (LADSPA-based)
- **URL:** [github.com/pulseaudio-equalizer-ladspa/equalizer](https://github.com/pulseaudio-equalizer-ladspa/equalizer)
- **Tech:** Python, Qt with LADSPA plugins
- **Status:** Unstable, may be deprecated
- **Limitations:** Crashes and audio artifacts reported, limited bands, no modern GUI

### Tier 3: Audio Plugin Frameworks

#### Calf Studio Gear
- **URL:** [github.com/calf-studio-gear/calf](https://github.com/calf-studio-gear/calf)
- **Tech:** C++, FLTK UI, LV2/JACK plugin format
- **Stars:** ~756
- **Status:** Actively maintained
- **Features:** 5/8/12/30-band graphic EQ, delay, reverb, compression, saturation, mastering effects
- **Limitations:** Requires JACK audio server, plugin-based (needs a host like Carla), not for casual users

#### Carla (Plugin Host)
- **URL:** [github.com/falkTX/Carla](https://github.com/falkTX/Carla)
- **Tech:** C++, Qt GUI
- **Status:** Actively maintained
- **Features:** Multi-format plugin host (LADSPA, DSSI, LV2, VST2, VST3), rack/patchbay modes
- **Limitations:** Complex interface, requires JACK or manual ALSA configuration

### Comparison Summary

| Project | Simplicity | UI Quality | Audio Quality | Casual User Ready |
|---------|-----------|------------|---------------|-------------------|
| **FX Sound (Windows)** | Excellent | Excellent | Excellent | Yes |
| **EasyEffects** | Poor | Good | Excellent | No |
| **JamesDSP** | Fair | Good | Very Good | No |
| **Calf Studio Gear** | Poor | Fair | Excellent | No |
| **PulseAudio EQ** | Fair | Poor | Fair | No |
| **Carla** | Poor | Good | Excellent | No |

---

## 4. Linux Audio Stack Overview

### Current State (2025-2026)

#### PipeWire — The Modern Standard
- **Fedora 34+** (2021): First major distro to adopt PipeWire by default
- **Ubuntu 22.10+**: Defaults to PipeWire
- **Debian 12 Bookworm**: PipeWire default for GNOME
- **Arch Linux**: Full PipeWire support
- **Pop!_OS 22.04+**: Switched to PipeWire

PipeWire unifies PulseAudio (consumer audio) and JACK (pro audio) into one system with low-latency support, better Bluetooth handling, and lower CPU usage.

#### How System-Wide Audio Processing Works on Linux
1. **Virtual Sink Approach:** Create a virtual audio sink that all applications output to. The sink processes audio through DSP effects before routing to the physical hardware device.
2. **PipeWire filter-chain module:** Allows creating arbitrary processing graphs from LADSPA, LV2, and built-in filters. Creates the virtual sink automatically.
3. **WirePlumber Smart Filters:** (PipeWire 1.x) Transparently inserts filters between streams and targets without creating a separate virtual sink.

#### Plugin Systems
| System | Status | Notes |
|--------|--------|-------|
| **LADSPA** | Legacy but widely supported | Simple API, PipeWire filter-chain supports it natively |
| **LV2** | Current standard | Modern, extensible, supports MIDI — overkill for our use case |
| **VST** | Limited Linux support | More Windows/macOS focused |

---

## 5. Recommended Technology Stack

### 5.1 Programming Language: Rust

| Reason | Detail |
|--------|--------|
| **Real-time safety** | No garbage collector — zero unpredictable latency in audio processing thread. GC pauses cause audio glitches (pops, crackles). Rust's ownership model provides memory safety at compile time with zero runtime overhead. |
| **Memory safety** | Ownership model prevents use-after-free, buffer overflows, data races at compile time |
| **Audio ecosystem** | Production crates: [CPAL](https://github.com/RustAudio/cpal), [DASP](https://github.com/RustAudio/dasp), [FunDSP](https://github.com/SamiPerttu/fundsp), [pipewire-rs](https://crates.io/crates/pipewire) |
| **Tauri backend** | Single language for entire non-UI codebase — no FFI boundaries within backend |
| **Performance** | Compiles to native code via LLVM with identical optimization passes as Clang/C++ |

**Why not C++:** While FX Sound itself uses C++/JUCE, C++ introduces memory safety risks (use-after-free, buffer overflows) that Rust eliminates at compile time. FxSound's DSP source code is available for algorithm study regardless of implementation language.

**Why not Python:** Unsuitable for real-time audio processing due to GIL and interpreter overhead. Good only for prototyping.

### 5.2 UI Framework: Tauri 2.0 + React + TypeScript

| Reason | Detail |
|--------|--------|
| **Beautiful UI** | Full CSS/SVG/Canvas/WebGL ecosystem for polished dark theme + smooth animations. Building FxSound-level aesthetics is dramatically easier with React + CSS than any native toolkit. |
| **Small binary** | ~600KB (vs ~150MB Electron). Uses system's WebKitGTK on Linux — no bundled browser engine. |
| **System tray** | Natively supported in Tauri 2.0 via `libappindicator` / `libayatana-appindicator` |
| **Rust backend** | DSP engine and PipeWire integration callable directly from application core |
| **Cross-desktop** | WebKitGTK renders consistently across GNOME, KDE, XFCE, Sway, Hyprland. Supports both X11 and Wayland. |
| **Visualization IPC** | 512-float FFT bins at 30fps (~60KB/s) — well within Tauri's Channel API limits using MessagePack binary serialization |

**Why not Qt:** More effort to achieve the polished, animated aesthetic. Qt licensing (LGPL/commercial) adds complexity. However, remains a viable fallback if WebKitGTK proves problematic.

**Why not GTK4:** GNOME-centric, feels foreign on KDE/XFCE. EasyEffects itself is migrating away from GTK4 to Qt/Kirigami in v8.0.

**Why not Electron:** ~150MB binary size, high memory usage. Overkill for this use case.

### 5.3 DSP Libraries

| Library | Purpose | License |
|---------|---------|---------|
| **[FunDSP](https://github.com/SamiPerttu/fundsp)** | Composable DSP graph framework. Built-in EQ filters (`bell_hz`, `lowshelf_hz`, `highshelf_hz`), reverb (`reverb_stereo`), compressor, stereo processing. `no_std` support for real-time contexts. Analytic frequency response queries (useful for EQ curve UI). | MIT/Apache-2.0 |
| **[biquad-rs](https://github.com/korken89/biquad-rs)** | Standalone IIR biquad filters with Direct Form 1 (ideal for online retuning — changing EQ parameters in real-time with minimal artifacts). `#![no_std]` support. | MIT/Apache-2.0 |
| **[RustFFT](https://crates.io/crates/rustfft)** | FFT for spectrum analyzer visualization. Pure Rust, version 5.0+ is faster than FFTW with AVX acceleration. | MIT/Apache-2.0 |
| **[DASP](https://github.com/RustAudio/dasp)** | PCM signal processing fundamentals — sample rate conversion, ring buffers, signal types. | MIT/Apache-2.0 |

### 5.4 Audio Backend: PipeWire (Hybrid Architecture)

#### Primary Path — LADSPA Plugin + PipeWire filter-chain
- DSP engine compiled as a LADSPA shared library using [ladspa.rs](https://github.com/nwoeanhinnogaehr/ladspa.rs) (MIT)
- PipeWire's `libpipewire-module-filter-chain` loads the plugin and creates the virtual sink
- GUI application generates/manages the filter-chain configuration file
- **Benefits:** DSP runs in PipeWire's own real-time thread with proper priority scheduling. Survives GUI crashes. Auto-recovers on PipeWire restart.

#### Secondary Path — Direct pw_filter (for v2.0)
- Use [pipewire-rs](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/) to create a `pw_filter` node directly
- Gives more programmatic control for per-app routing (v2.0 feature)

**Why LADSPA over LV2:** LADSPA is simpler to implement, PipeWire filter-chain supports it natively, and it's sufficient for audio effects with control parameters (no MIDI needed). LV2 adds unnecessary complexity.

### 5.5 Frontend Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| React | 19 | UI component framework |
| TypeScript | 5 | Type-safe frontend code |
| Vite | 6 | Fast build tool and dev server |
| Tailwind CSS | 4 | Utility-first styling for rapid dark theme development |
| Canvas 2D / WebGL | — | Spectrum analyzer and EQ curve visualization |

### 5.6 Build & Distribution

| Tool | Purpose |
|------|---------|
| Cargo | Rust build system and dependency management |
| Vite | Frontend bundler |
| Tauri CLI | Application packaging |
| cargo-deb | .deb package generation (Ubuntu/Debian) |
| cargo-rpm | .rpm package generation (Fedora/RHEL) |
| tauri-action | GitHub Actions CI/CD for AppImage, .deb, .rpm |
| AUR PKGBUILD | Arch Linux package |

---

## 6. System Architecture

### 6.1 High-Level Component Diagram

```
+------------------------------------------------------------------+
|                        FxSonic Application                        |
|                                                                   |
|  +---------------------------+   +-----------------------------+  |
|  |     Tauri Backend (Rust)  |   |   React Frontend (TS/JS)   |  |
|  |                           |   |                             |  |
|  |  - Settings Manager       |   |  - PowerToggle (animated)  |  |
|  |    (TOML config files)    |   |  - 5 Effect Sliders        |  |
|  |                           |   |  - EQ Curve Canvas (SVG)   |  |
|  |  - PipeWire Manager       |   |  - Spectrum Visualizer     |  |
|  |    (pipewire-rs)          |   |    (Canvas 2D, 30fps)      |  |
|  |    - Device detection     |<->|  - Preset Selector         |  |
|  |    - Default sink mgmt   | IPC|  - Device Selector          |  |
|  |    - Graph monitoring     |   |  - System Tray Menu        |  |
|  |                           |   |                             |  |
|  |  - DSP Controller         |   |                             |  |
|  |    - Parameter bridge     |   |                             |  |
|  |    - FFT analyzer         |   |                             |  |
|  |    - Preset loader        |   |                             |  |
|  +---------------------------+   +-----------------------------+  |
+------------------------------------------------------------------+
         |                    |
         | Manages config     | Reads FFT data from shared memory
         v                    v
+------------------+   +---------------------------+
| PipeWire Daemon  |   | LADSPA Plugin             |
|                  |   | (libfxsonic_dsp.so)       |
| filter-chain     |-->|                           |
| module loads     |   | [Bass Boost]              |
| LADSPA plugin    |   | [Clarity]                 |
|                  |   | [Ambiance]                |
| Virtual Sink     |   | [Surround Sound]          |
| "FxSonic"        |   | [Dynamic Boost]           |
|   ^              |   | [10-Band Parametric EQ]   |
|   |              |   | [Output Limiter]          |
|   | Default      |   |                           |
|   | Output       |   +---------------------------+
|   |              |
+---+--------------+
    |
    | All applications'
    | audio flows here
```

### 6.2 Audio Processing Pipeline

```
Input Signal (32-bit float, stereo)
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ [1] Bass Boost                                          │
│     Low-shelf filter + harmonic excitation               │
│     Center: ~80-120Hz, adjustable boost 0-12dB          │
├─────────────────────────────────────────────────────────┤
│ [2] Clarity                                             │
│     Dynamic high-shelf + presence boost                  │
│     Shelf at ~4kHz, gain modulated by signal envelope   │
│     Enhances transients and harmonic content             │
├─────────────────────────────────────────────────────────┤
│ [3] Ambiance                                            │
│     Algorithmic reverb (Freeverb/Schroeder topology)    │
│     Wet/dry mix controlled by slider (0-100%)           │
│     Parameters: pre-delay, room size, damping           │
├─────────────────────────────────────────────────────────┤
│ [4] Surround Sound                                      │
│     Mid-side stereo widening                             │
│     M = (L+R)/2, S = (L-R)/2                           │
│     Boost S channel + Haas effect (short delay)         │
├─────────────────────────────────────────────────────────┤
│ [5] Dynamic Boost                                       │
│     3-band multiband compressor                         │
│     Low: <250Hz, Mid: 250-4kHz, High: >4kHz            │
│     Gentle ratio (2:1-4:1), auto makeup gain            │
├─────────────────────────────────────────────────────────┤
│ [6] 10-Band Parametric EQ                               │
│     Bell/peaking filters                                 │
│     Default: 31, 62, 125, 250, 500, 1k, 2k, 4k, 8k,   │
│     16kHz — each band: ±12dB, adjustable Q              │
├─────────────────────────────────────────────────────────┤
│ [7] Output Limiter                                      │
│     Brick-wall limiter at -0.3dBFS                      │
│     Prevents clipping from accumulated boost            │
└─────────────────────────────────────────────────────────┘
    │
    ▼
Output Signal (32-bit float, stereo)
```

### 6.3 IPC Data Flow

```
Rust Backend                    IPC Channel               React Frontend
┌──────────────────┐    Binary (MessagePack)     ┌────────────────────┐
│ FFT Analyzer     │ ─────── 30fps ────────────> │ Spectrum Canvas    │
│ (512-bin output) │                             │ (requestAnimFrame) │
└──────────────────┘                             └────────────────────┘

┌──────────────────┐    Tauri Command (JSON)     ┌────────────────────┐
│ DSP Controller   │ <───── set_effect() ─────── │ Effect Sliders     │
│                  │ <───── set_eq_band() ────── │ EQ Curve Editor    │
│                  │ <───── load_preset() ────── │ Preset Selector    │
│                  │ <───── toggle_power() ───── │ Power Button       │
└──────────────────┘                             └────────────────────┘

┌──────────────────┐    Tauri Command (JSON)     ┌────────────────────┐
│ PipeWire Manager │ ──── get_devices() ───────> │ Device Selector    │
│                  │ ──── get_state() ──────────>│ Status Indicator   │
└──────────────────┘                             └────────────────────┘
```

### 6.4 File System Layout

```
~/.config/fxsonic/
├── config.toml                    # User settings (last state, window position, preferences)
└── presets/
    ├── general.toml               # Built-in presets (copied on first run)
    ├── music.toml
    ├── voice.toml
    ├── streaming.toml
    ├── bass-boost.toml
    └── custom/                    # User-created presets (v2.0)

~/.config/pipewire/pipewire.conf.d/
└── fxsonic-filter.conf            # Generated PipeWire filter-chain config

/usr/lib/ladspa/
└── libfxsonic_dsp.so              # LADSPA plugin (installed by package)

/usr/share/applications/
└── fxsonic.desktop                # Desktop entry

/usr/share/icons/hicolor/...
└── fxsonic.png                    # App icon (multiple sizes)
```

### 6.5 PipeWire Filter-Chain Configuration (Generated)

```conf
# ~/.config/pipewire/pipewire.conf.d/fxsonic-filter.conf
context.modules = [
    {   name = libpipewire-module-filter-chain
        args = {
            node.description = "FxSonic Audio Enhancer"
            media.name       = "FxSonic"
            filter.graph = {
                nodes = [
                    {
                        type   = ladspa
                        name   = fxsonic
                        plugin = libfxsonic_dsp
                        label  = fxsonic_enhancer
                        control = {
                            "Bass"          = 0.5
                            "Clarity"       = 0.5
                            "Ambiance"      = 0.3
                            "Surround"      = 0.4
                            "Dynamic Boost" = 0.5
                            "EQ Band 1"     = 0.0
                            "EQ Band 2"     = 0.0
                            ...
                            "Enabled"       = 1.0
                        }
                    }
                ]
            }
            capture.props = {
                node.name      = "fxsonic_sink"
                media.class    = Audio/Sink
                audio.position = [ FL FR ]
            }
            playback.props = {
                node.name      = "fxsonic_playback"
                node.passive   = true
                audio.position = [ FL FR ]
            }
        }
    }
]
```

---

## 7. Feature Requirements

### 7.1 MVP (v1.0) — Core Experience

| Category | Feature | Priority | Notes |
|----------|---------|----------|-------|
| **Audio Engine** | PipeWire virtual sink creation | P0 | System-wide capture via LADSPA + filter-chain |
| **Audio Engine** | Real-time DSP pipeline | P0 | 32-bit float, < 10ms latency |
| **Audio Engine** | Bass Boost effect | P0 | Low-shelf filter + harmonic exciter |
| **Audio Engine** | Clarity effect | P0 | Dynamic high-shelf, envelope follower |
| **Audio Engine** | Ambiance effect | P0 | Algorithmic reverb (Freeverb) |
| **Audio Engine** | Surround Sound effect | P0 | Mid-side stereo widening + Haas delay |
| **Audio Engine** | Dynamic Boost effect | P0 | 3-band multiband compressor |
| **Audio Engine** | 10-band parametric EQ | P0 | Adjustable frequency, gain (±12dB), Q per band |
| **Audio Engine** | Output limiter | P0 | Brick-wall at -0.3dBFS |
| **Audio Engine** | Master on/off toggle | P0 | Clean bypass (zero processing when off) |
| **UI** | Dark theme main window | P0 | FxSound-inspired: dark background, red accents |
| **UI** | 5 effect sliders | P0 | Custom styled, smooth animations |
| **UI** | EQ curve display | P0 | Interactive SVG/Canvas frequency response |
| **UI** | Real-time spectrum visualizer | P1 | FFT-based, 30fps, Canvas 2D |
| **UI** | Preset selector | P0 | Dropdown with 5 built-in presets |
| **UI** | Power toggle button | P0 | Animated on/off with visual feedback |
| **System** | System tray icon | P0 | Show/hide, quick preset switch, quit |
| **System** | Settings persistence | P0 | Remember all settings across restarts (TOML) |
| **System** | Auto-start on login | P1 | XDG autostart desktop entry |
| **System** | PipeWire reconnection | P1 | Handle PipeWire restarts gracefully |

### 7.2 v2.0 — Enhanced Experience

| Feature | Notes |
|---------|-------|
| Per-application audio routing | Route specific apps through FxSonic |
| Microphone (input) processing | Voice enhancement for calls |
| Mini-mode compact view | Small floating window with essential controls |
| Custom preset creation/management | Save, rename, delete, import/export |
| Genre-specific presets | Rock, Jazz, Classical, Hip-Hop, EDM, etc. |
| Global keyboard shortcuts | Toggle power, switch presets without opening UI |
| Multi-output device profiles | Different settings per audio device |
| Flatpak/Snap packaging | Broader distribution |

### 7.3 v3.0 — Advanced

| Feature | Notes |
|---------|-------|
| Convolution reverb | Load custom impulse response files |
| Loudness normalization (EBU R128) | Consistent volume across all apps |
| Crossfeed for headphones | Reduce ear fatigue |
| Theme customization | Accent colors, light mode option |
| Spectrogram view | Frequency-over-time visualization |
| PulseAudio fallback mode | Support for older distros |
| CLI interface | Headless/scripting control |

---

## 8. Key Technical Challenges & Solutions

### Challenge 1: Real-Time Safety in DSP Callback

**Problem:** PipeWire's process callback runs on a real-time thread. Any heap allocation, mutex lock, file I/O, or system call causes audio glitches (pops, crackles, dropouts).

**Solution:**
- All DSP uses stack-allocated or pre-allocated buffers
- FunDSP's graph nodes are `no_std` compatible and allocation-free in the process path
- Parameter updates use **lock-free ring buffers** (`ringbuf` crate or `crossbeam-channel` with bounded capacity)
- LADSPA control port mechanism already provides a real-time-safe parameter interface

### Challenge 2: Tauri IPC Bandwidth for Visualization

**Problem:** Sending FFT data at 30-60fps from Rust to JavaScript could bottleneck on JSON serialization.

**Solution:**
- Use Tauri's **Channel API** for streaming data
- Serialize FFT data as **MessagePack** binary (via `rmp-serde`) — ~5x faster than JSON
- Send only 512 float values (2KB) per frame at 30fps = ~60KB/s total
- Downsample to logarithmic frequency bins matching visualization bar count (e.g., 64 bars)
- Fallback: local WebSocket connection bypassing Tauri IPC if needed

### Challenge 3: Virtual Sink as Default Audio Output

**Problem:** When FxSonic creates a virtual sink, all applications need to route audio through it automatically.

**Solution:**
- Use `pw-metadata` or pipewire-rs registry API to set FxSonic sink as default
- On exit, restore the original default sink
- Monitor PipeWire registry events and re-route if user manually changes default
- Alternative: WirePlumber Smart Filters (PipeWire 1.x) — transparently inserts filter without separate virtual sink

### Challenge 4: Surviving PipeWire Restarts

**Problem:** PipeWire can restart (sleep/wake, updates). Application must reconnect gracefully.

**Solution:**
- LADSPA plugin approach auto-recovers — loaded by PipeWire config, restarts with PipeWire
- GUI reconnects via pipewire-rs with exponential backoff retry loop
- Monitor connection state via Core proxy events

### Challenge 5: Cross-Desktop Compatibility

**Problem:** GNOME, KDE, XFCE, Sway, Hyprland all have different system tray protocols and theme engines.

**Solution:**
- Tauri 2.0 system tray uses `libappindicator` — works on GNOME (with extension), KDE, XFCE, most tiling WMs
- WebKitGTK renders consistently across all desktops
- Wayland support via WebKitGTK's native Wayland backend
- Test targets: Ubuntu 24.04 (GNOME), Fedora 41 (GNOME), KDE Neon, Arch + Hyprland

### Challenge 6: Matching FX Sound's Audio Quality

**Problem:** FxSound's DSP has been tuned over 25+ years. Generic implementations won't match.

**Solution:**
- **Study the FxSound DfxDsp source code** ([fxsound2/fxsound-app `dsp/`](https://github.com/fxsound2/fxsound-app)) — analyze filter topologies, coefficients, and processing chains
- Bass Boost: FunDSP `lowshelf_hz()` + soft-saturation waveshaper for warmth
- Clarity: Sidechain-controlled high-shelf — boost modulated by high-frequency envelope
- Ambiance: Tuned Freeverb via FunDSP `reverb_stereo()` with calibrated pre-delay/room/damping
- Surround: Mid-side processing + subtle Haas effect (keep < 5ms to avoid phase issues)
- Dynamic Boost: 3-band compressor with Linkwitz-Riley crossovers, gentle ratios

---

## 9. Dependency Manifest

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
# Core application
tauri = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }

# Audio / PipeWire
pipewire = "0.9"          # PipeWire Rust bindings
ladspa = "0.2"            # LADSPA plugin interface

# DSP
fundsp = "0.18"           # DSP graph framework
biquad = "0.4"            # IIR biquad filters
rustfft = "6"             # FFT for visualization
dasp = "0.11"             # PCM signal processing fundamentals

# Serialization (for visualization data IPC)
rmp-serde = "1"           # MessagePack serialization

# Configuration
directories = "5"         # XDG base directory paths
toml = "0.8"              # Config file format
```

### Frontend Dependencies (package.json)

```json
{
  "dependencies": {
    "react": "^19",
    "react-dom": "^19",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2"
  },
  "devDependencies": {
    "typescript": "^5",
    "vite": "^6",
    "@vitejs/plugin-react": "^4",
    "tailwindcss": "^4"
  }
}
```

### System Dependencies

| Package | Purpose | Required |
|---------|---------|----------|
| `pipewire` | Audio server | Yes (runtime) |
| `pipewire-pulse` | PulseAudio compatibility | Recommended |
| `libwebkit2gtk-4.1` | Tauri WebView rendering | Yes (runtime) |
| `libappindicator3` or `libayatana-appindicator` | System tray | Yes (runtime) |
| `rustup` / `cargo` | Rust toolchain | Yes (build) |
| `node` + `npm` | Frontend build | Yes (build) |
| `pkg-config` | Library discovery | Yes (build) |
| `libpipewire-dev` | PipeWire headers | Yes (build) |
| `libclang-dev` | Rust bindgen | Yes (build) |

---

## 10. Development Phases

### Phase 0: Foundation (Weeks 1-3)

**Goal:** Prove the core audio pipeline works end-to-end.

- [ ] Set up Rust workspace: `fxsonic-dsp` (LADSPA plugin crate) + `fxsonic-app` (Tauri app crate)
- [ ] Implement minimal LADSPA plugin — identity transform (passthrough audio)
- [ ] Write PipeWire filter-chain config to load the plugin as a virtual sink
- [ ] Verify: play music, confirm audio flows through plugin and out speakers
- [ ] Scaffold Tauri 2.0 + React + TypeScript project
- [ ] Implement basic Rust↔JS IPC: a "get PipeWire state" command

**Deliverable:** Audio passes through a Rust LADSPA plugin loaded by PipeWire. Tauri window shows "Connected to PipeWire."

### Phase 1: DSP Engine (Weeks 4-8)

**Goal:** All 5 effects + parametric EQ working and testable.

- [ ] Implement Bass Boost (low-shelf + harmonic exciter)
- [ ] Implement Clarity (dynamic high-shelf, envelope follower)
- [ ] Implement Ambiance (Freeverb with wet/dry control)
- [ ] Implement Surround Sound (mid-side + Haas delay)
- [ ] Implement Dynamic Boost (3-band compressor)
- [ ] Implement 10-band parametric EQ (FunDSP `bell_hz()` filters)
- [ ] Add output limiter (brick-wall at -0.3dBFS)
- [ ] Expose all parameters as LADSPA control ports
- [ ] Unit tests: verify each effect with known test signals
- [ ] Benchmark: < 5ms processing latency at 48kHz/256 samples

**Deliverable:** LADSPA plugin with all effects, controllable via control ports, tested.

### Phase 2: User Interface (Weeks 9-14)

**Goal:** Beautiful, fully functional UI.

- [ ] Design main window (dark theme, red accents, FxSound-inspired layout)
- [ ] Power toggle with animation
- [ ] 5 custom-styled effect sliders
- [ ] EQ curve canvas (SVG/Canvas 2D) — interactive click/drag
- [ ] Real-time spectrum analyzer (Canvas 2D, `requestAnimationFrame`)
- [ ] Preset selector dropdown (5 presets)
- [ ] Tauri commands: `set_effect`, `set_eq_band`, `toggle_power`, `load_preset`, `get_spectrum_data`
- [ ] Settings persistence (TOML at `~/.config/fxsonic/config.toml`)
- [ ] System tray: icon, show/hide, quick preset switch, quit
- [ ] Device selector dropdown

**Deliverable:** Fully functional application with all effects controllable through polished UI.

### Phase 3: Polish & Distribution (Weeks 15-18)

**Goal:** Production quality, tested, and packaged.

- [ ] Animation polish: transitions, micro-interactions, hover effects
- [ ] Performance profiling and DSP optimization
- [ ] Error handling: PipeWire disconnect recovery, missing config, permissions
- [ ] Auto-start: XDG `.desktop` file in `~/.config/autostart/`
- [ ] Packaging: AppImage, .deb, .rpm, AUR PKGBUILD
- [ ] GitHub Actions CI/CD for automated builds
- [ ] Testing on Ubuntu 24.04, Fedora 41, Arch Linux, Debian 12
- [ ] README, installation guide, user documentation

**Deliverable:** v1.0 release with packages for all major distros.

### Phase 4: v2.0 Features (Months 5-8)

- [ ] Per-application audio routing
- [ ] Microphone processing mode
- [ ] Mini-mode compact view
- [ ] Custom preset creation/management
- [ ] Global keyboard shortcuts
- [ ] Flatpak package

---

## 11. Resources to Leverage

### Code References

| Resource | What to Extract | License |
|----------|----------------|---------|
| [fxsound2/fxsound-app `dsp/`](https://github.com/fxsound2/fxsound-app) | DSP algorithm topology, filter coefficients, effect chain order, preset format | AGPL-3.0 |
| [FunDSP `hacker.rs`](https://github.com/SamiPerttu/fundsp/blob/master/src/hacker.rs) | EQ filters, reverb, compressor, stereo processing, pipe operator composition | MIT/Apache-2.0 |
| [ladspa.rs](https://github.com/nwoeanhinnogaehr/ladspa.rs) | LADSPA plugin boilerplate, port definition, descriptor setup | MIT |
| [pipewire-rs](https://crates.io/crates/pipewire) | PipeWire connection, registry monitoring, device enumeration | MIT |
| [biquad-rs](https://github.com/korken89/biquad-rs) | Real-time-safe IIR filters with DF1 online retuning | MIT/Apache-2.0 |
| [EasyEffects](https://github.com/wwmm/easyeffects) | PipeWire virtual sink integration patterns, stream routing logic | GPL-3.0 |
| [Airwindows](https://github.com/airwindows/airwindows) | 600+ open-source C++ effect implementations for algorithm reference | MIT |
| [Audio EQ Cookbook](https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html) | Canonical biquad coefficient formulas for all filter types | Public |
| [PipeWire filter-chain docs](https://docs.pipewire.org/page_module_filter_chain.html) | Configuration format reference | MIT |
| [Freeverb algorithm (CCRMA)](https://ccrma.stanford.edu/~jos/pasp/Freeverb.html) | Schroeder reverb topology for the Ambiance effect | Academic |

### UI/Design References

| Resource | What to Extract |
|----------|----------------|
| FX Sound Windows app screenshots | Color palette, slider design, layout proportions, visual hierarchy |
| [audiomotion-analyzer](https://github.com/hvianna/audiomotion-analyzer) | High-performance web spectrum analyzer component |
| [react-audio-visualize](https://www.npmjs.com/package/react-audio-visualize) | Audio visualization React components |

---

## 12. Verification Plan

| Test | Method | Success Criteria |
|------|--------|-----------------|
| **Audio pipeline** | Play music through any app | Audio routes through FxSonic virtual sink, effects audible |
| **Effect quality** | A/B compare against FX Sound on Windows | Comparable audio enhancement quality |
| **Latency** | Measure with `pw-top` | < 10ms at 48kHz / 256 samples |
| **UI responsiveness** | Move sliders rapidly | Parameters update in real-time, no lag |
| **EQ curve** | Adjust bands, verify visual matches audio | Curve accurately reflects frequency response |
| **Visualizer sync** | Play various content | Spectrum display synced with audio |
| **Presets** | Load each preset | All effect values match expected configuration |
| **System tray** | Test on GNOME, KDE, tiling WM | Show/hide, preset switch, quit all work |
| **Persistence** | Change settings → close → reopen | All settings restored exactly |
| **PipeWire recovery** | `systemctl --user restart pipewire` | Audio resumes without app restart |
| **Distro compatibility** | Install and run on 4 distros | Works on Ubuntu 24.04, Fedora 41, Arch, Debian 12 |
| **Resource usage** | Monitor with `htop` during playback | CPU < 5% on modern hardware, RAM < 100MB |

---

## 13. Summary — Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | **Rust** | Memory safety + real-time performance + Tauri integration |
| UI Framework | **Tauri 2.0 + React + TypeScript** | Beautiful UI via web tech, tiny binary, Rust backend |
| DSP Framework | **FunDSP + biquad-rs** | Composable graph DSP, real-time safe, analytic frequency response |
| Audio Integration | **LADSPA plugin + PipeWire filter-chain** | Most stable; DSP runs in PipeWire's RT thread; survives GUI crashes |
| Parameter Passing | **LADSPA control ports** | Real-time safe, no mutexes in audio path |
| Visualization IPC | **Tauri Channel + MessagePack** | Low overhead for 30fps FFT data stream |
| Config Format | **TOML** | Human-readable, excellent Rust ecosystem support |
| Distribution | **AppImage + .deb + .rpm + AUR** | Covers 95%+ of desktop Linux users |
