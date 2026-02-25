● Zero errors. The system is fully operational. Here's a summary of what was fixed and how things work
  now:

  What was wrong

  1. LADSPA plugin failing to start — The old PipeWire filter-chain config was missing the explicit
  inputs/outputs graph mapping and audio.rate/audio.channels properties. This caused spa.audioadapter: no
  matching params → error -95: Operation not supported.
  2. Audio not routing through FxSonic — The FxSonic virtual sink was never set as the default audio
  output, so all apps (Brave, etc.) bypassed it entirely and went directly to the hardware speakers.
  3. Config changes had no effect — Writing to ~/.config/pipewire/pipewire.conf.d/ only takes effect at
  PipeWire startup. There was no mechanism to reload the filter-chain at runtime.

  New Architecture

  1. Standalone filter-chain process — Instead of using PipeWire's conf.d, the app spawns pipewire -c
  <config> as a child process. This connects to the main PipeWire daemon and creates the filter-chain
  nodes.
  2. Effect/EQ changes — Frontend debounces slider changes (600ms). After the user stops adjusting, the
  app kills the old filter process, writes a new config with updated values, and spawns a new one.
  3. Device switching — The selected device is written as target.object in the config. Changing the device
   restarts the filter process, which automatically routes to the new device.
  4. FxSonic as default sink — WirePlumber remembers the default sink preference. After the first run sets
   it, subsequent restarts auto-apply.

  Audio Chain

  Apps → fxsonic_sink (virtual) → LADSPA DSP processing → fxsonic_playback → Selected Device

  Play some audio in your browser to test it. You should hear it through the FANTECH CAPTAIN 7.1 (your
  currently selected device). Try changing effects — after you stop adjusting, there'll be a brief moment
  while the filter restarts, then the new effect values will be active.
