# FxSonic

FxSonic is a Linux audio enhancement application inspired by FxSound, providing five main effect sliders with a simple, beautiful interface.

## Features

- **5 Main Effects**: Bass Boost, Clarity, Ambiance, Surround Sound, Dynamic Boost
- **10-Band Parametric EQ**: Fine-tune your audio with precision
- **Real-time Visualization**: Spectrum analyzer and EQ curve display
- **Simple Interface**: Just a power button and sliders - no complex configuration needed
- **System-wide Audio**: Enhances audio from all applications automatically
- **PipeWire Integration**: Modern Linux audio stack support

## Technology Stack

- **Backend**: Rust (Tauri 2.0)
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS
- **Audio Processing**: FunDSP + LADSPA
- **Audio Backend**: PipeWire

## Prerequisites

Before building FxSonic, ensure you have the following installed:

### Required Tools
- Rust toolchain (rustup)
- Node.js and npm
- PipeWire (running)

### System Dependencies (Required!)

**⚠️ Important:** You must install these system dependencies before building. The build will fail without them.

**Ubuntu/Debian:**
```bash
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
```

**Fedora:**
```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    glib2-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    pipewire-devel \
    clang-devel \
    pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S --needed \
    webkit2gtk-4.1 \
    glib2 \
    gtk3 \
    libayatana-appindicator \
    librsvg \
    pipewire \
    clang \
    pkg-config \
    base-devel
```

## Building from Source

### 1. Clone the repository
```bash
git clone https://github.com/fxsonic/fxsonic.git
cd fxsonic
```

### 2. Install Rust (if not already installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install system dependencies (Required!)
See the **Prerequisites** section above for your distribution.

### 4. Install frontend dependencies
```bash
cd fxsonic-app/frontend
npm install
cd ../..
```

### 5. Build the Rust workspace
```bash
cargo build --workspace
```

### 6. Build the application
```bash
cd fxsonic-app/frontend
npm run tauri build
```

The built application will be in `fxsonic-app/src-tauri/target/release/bundle/`.

## Development

### Running in development mode
```bash
cd fxsonic-app/frontend
npm install
npm run tauri dev
```

This will start the Vite dev server and launch the Tauri application with hot reload.

### Project Structure

```
fxsonic/
├── fxsonic-dsp/              # LADSPA plugin (audio processing)
│   ├── src/
│   │   └── lib.rs           # DSP effects implementation
│   └── Cargo.toml
├── fxsonic-app/              # Tauri application
│   ├── src/
│   │   └── main.rs          # Rust backend + IPC handlers
│   ├── src-tauri/
│   │   ├── tauri.conf.json  # Tauri configuration
│   │   └── build.rs
│   ├── frontend/            # React frontend
│   │   ├── src/
│   │   │   ├── App.tsx      # Main UI component
│   │   │   └── main.tsx
│   │   ├── package.json
│   │   ├── tailwind.config.js
│   │   └── vite.config.ts
│   └── Cargo.toml
└── Cargo.toml               # Workspace configuration
```

## Usage

1. Launch FxSonic
2. The application will automatically connect to PipeWire
3. Click the power button to enable/disable audio enhancement
4. Adjust the effect sliders to customize your audio
5. Select a preset for quick configuration

## Effects

### Bass Boost
Enhances low frequencies for deeper, punchier bass.

### Clarity
Improves high-frequency detail for crisp, articulate audio.

### Ambiance
Adds reverb to simulate larger spaces like concert halls.

### Surround Sound
Widens the stereo field for a more immersive experience.

### Dynamic Boost
Compresses audio to restore lost dynamic range.

## Presets

- **General**: Balanced enhancement for everyday use
- **Music**: Optimized for music playback
- **Voice**: Enhances speech clarity
- **Streaming Video**: Optimized for video content
- **Bass Boost**: Maximum bass enhancement

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Inspired by [FxSound](https://github.com/fxsound2/fxsound-app)
- Built with [Tauri](https://tauri.app/)
- DSP powered by [FunDSP](https://github.com/SamiPerttu/fundsp)
