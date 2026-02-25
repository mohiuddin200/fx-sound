# FxSonic Project Initialization Tasks

This document outlines all tasks required to initialize the FxSonic project from scratch.

## Prerequisites Verification

Before starting initialization, verify your system has the required tools:

- [ ] Rust toolchain (`rustup` and `cargo`)
- [ ] Node.js and npm (for frontend)
- [ ] PipeWire installed and running
- [ ] System dependencies: `pkg-config`, `libpipewire-dev`, `libwebkit2gtk-4.1`, `libclang-dev`

## Step 1: Project Structure Setup

- [ ] Create Rust workspace with two crates:
  - [ ] `fxsonic-dsp/` - LADSPA plugin library
  - [ ] `fxsonic-app/` - Tauri application
- [ ] Initialize Cargo workspace in root `Cargo.toml`
- [ ] Create frontend directory structure
- [ ] Set up initial `.gitignore` file
- [ ] Create README with project overview

## Step 2: Rust Backend Initialization

### fxsonic-dsp crate (LADSPA Plugin)

- [ ] Initialize `fxsonic-dsp/Cargo.toml` with dependencies:
  - [ ] `ladspa` v0.2
  - [ ] `fundsp` v0.18
  - [ ] `biquad` v0.4
  - [ ] `rustfft` v6
  - [ ] `dasp` v0.11
- [ ] Create `src/lib.rs` with LADSPA plugin descriptor structure
- [ ] Implement minimal passthrough audio processing (identity transform)
- [ ] Add LADSPA control port definitions for future effects
- [ ] Build as shared library (`cdylib` target)
- [ ] Verify `.so` file is generated in `target/release/`

### fxsonic-app crate (Tauri Backend)

- [ ] Initialize Tauri 2.0 project structure
- [ ] Create `fxsonic-app/Cargo.toml` with dependencies:
  - [ ] `tauri` v2
  - [ ] `serde` (with derive feature)
  - [ ] `serde_json`
  - [ ] `tokio` (with full features)
  - [ ] `pipewire` v0.9
  - [ ] `rmp-serde` v1 (MessagePack serialization)
  - [ ] `directories` v5
  - [ ] `toml` v0.8
- [ ] Create `src/main.rs` with basic Tauri app boilerplate
- [ ] Set up Tauri IPC command handlers
- [ ] Implement basic PipeWire connection check

## Step 3: Frontend Initialization (React + TypeScript)

- [ ] Initialize `package.json` with dependencies:
  - [ ] `react` v19
  - [ ] `react-dom` v19
  - [ ] `@tauri-apps/api` v2
  - [ ] `@tauri-apps/plugin-shell` v2
- [ ] Initialize `package.json` devDependencies:
  - [ ] `typescript` v5
  - [ ] `vite` v6
  - [ ] `@vitejs/plugin-react` v4
  - [ ] `tailwindcss` v4
- [ ] Create `vite.config.ts` configuration
- [ ] Create `tsconfig.json` configuration
- [ ] Initialize Tailwind CSS configuration
- [ ] Set up `src/main.tsx` as React entry point
- [ ] Create `src/App.tsx` with basic window structure

## Step 4: Tauri Configuration

- [ ] Create `fxsonic-app/src-tauri/tauri.conf.json`
- [ ] Configure app metadata:
  - [ ] App name: "FxSonic"
  - [ ] Window title
  - [ ] Dark theme colors (background: `#1a1a2e`, accent: `#e94560`)
- [ ] Set up system tray configuration
- [ ] Configure build targets (Linux)
- [ ] Set up permissions (allowlist)

## Step 5: PipeWire Integration Setup

- [ ] Create PipeWire filter-chain configuration template
- [ ] Design configuration file structure for `~/.config/pipewire/pipewire.conf.d/fxsonic-filter.conf`
- [ ] Create Rust module for PipeWire management:
  - [ ] Device detection
  - [ ] Default sink management
  - [ ] Filter-chain config generation
- [ ] Implement basic PipeWire connection test

## Step 6: Basic UI Implementation

### Main Window Structure

- [ ] Create dark-themed main layout
- [ ] Add power toggle button (placeholder)
- [ ] Add 5 effect sliders (placeholder components)
- [ ] Add EQ curve canvas placeholder
- [ ] Add spectrum analyzer placeholder
- [ ] Add preset selector dropdown placeholder
- [ ] Add device selector placeholder

### Styling

- [ ] Set up Tailwind CSS base styles
- [ ] Create custom color palette matching FxSound
- [ ] Implement responsive layout
- [ ] Add dark theme CSS variables

## Step 7: IPC Communication Setup

- [ ] Define Tauri commands in Rust backend:
  - [ ] `get_pipewire_state()` - Return PipeWire connection status
  - [ ] `set_effect()` - Set effect parameter (placeholder)
  - [ ] `toggle_power()` - Toggle master on/off (placeholder)
- [ ] Create Tauri command bindings in TypeScript
- [ ] Test basic IPC communication (ping/pong)

## Step 8: DSP Engine Foundation

- [ ] Design effect pipeline architecture in `fxsonic-dsp`
- [ ] Implement lock-free parameter update mechanism
- [ ] Create placeholder control ports for:
  - [ ] Bass Boost
  - [ ] Clarity
  - [ Ambiance
  - [ ] Surround Sound
  - [ ] Dynamic Boost
  - [ ] 10 EQ bands
  - [ ] Master enable/disable
- [ ] Set up DSP graph structure using FunDSP

## Step 9: Build System Integration

- [ ] Configure build scripts to compile DSP plugin
- [ ] Set up Tauri build to bundle DSP `.so` file
- [ ] Create build script for generating PipeWire config
- [ ] Test full build process: `npm run tauri build`

## Step 10: Configuration Management

- [ ] Create XDG config directory structure
- [ ] Design TOML config file schema:
  - [ ] Last state persistence
  - [ ] Effect values
  - [ ] EQ band values
  - [ ] Window position/size
  - [ ] Last selected preset
- [ ] Implement config file read/write in Rust
- [ ] Create preset directory structure

## Step 11: Testing Foundation

- [ ] Set up basic unit tests for DSP functions
- [ ] Create integration test for PipeWire connection
- [ ] Test audio passthrough through LADSPA plugin
- [ ] Verify: play music, confirm audio flows through plugin

## Step 12: Documentation

- [ ] Create `README.md` with:
  - [ ] Project overview
  - [ ] Build instructions
  - [ ] Installation guide
  - [ ] Development setup
- [ ] Document project structure
- [ ] Document IPC command API
- [ ] Document LADSPA plugin interface

## Verification Checklist

Once initialization is complete, verify:

- [ ] `cargo build --release` succeeds for workspace
- [ ] `npm run tauri dev` launches the app
- [ ] App window opens with dark theme
- [ ] PipeWire connection status shows in UI
- [ ] DSP plugin `.so` file is generated
- [ ] Tauri IPC commands respond correctly
- [ ] Config directory structure is created

## Next Steps After Initialization

After completing all initialization tasks, proceed to:

1. **Phase 1:** Implement full DSP effects (Bass Boost, Clarity, Ambiance, Surround, Dynamic Boost, EQ)
2. **Phase 2:** Build complete UI with real sliders, visualizers, and interactions
3. **Phase 3:** Polish animations, handle errors, package for distribution

---

**Notes:**
- Follow the existing codebase conventions as you implement
- All DSP code must be real-time safe (no allocations in audio thread)
- Use lock-free structures for parameter updates
- Test on multiple distributions when possible (Ubuntu, Fedora, Arch)
