# Contributing to FxSonic

Thank you for your interest in contributing to FxSonic!

## Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## Coding Standards

- Follow Rust's official style guide (`cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and small
- Write tests for new functionality

## Testing

Before submitting a PR, ensure:
- `cargo test` passes
- `npm run tauri build` succeeds
- Manual testing of your changes
- Audio still passes through correctly

## DSP Guidelines

- All audio processing must be real-time safe
- No allocations in the audio thread
- Use lock-free structures for parameter updates
- Test with various audio sources

## UI/UX Guidelines

- Maintain the dark theme with red accents
- Keep the interface simple and intuitive
- Ensure responsive design
- Test on different desktop environments (GNOME, KDE, XFCE)

## Issue Reporting

When reporting bugs, please include:
- Linux distribution and version
- PipeWire version
- Steps to reproduce
- Expected vs actual behavior
- Any relevant logs
