import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

type EffectSetting = {
  enabled: boolean;
  value: number;
};

type EqBandSetting = {
  frequency: number;
  gain: number;
  q: number;
};

type Settings = {
  bass_boost: EffectSetting;
  clarity: EffectSetting;
  ambiance: EffectSetting;
  surround: EffectSetting;
  dynamic_boost: EffectSetting;
  eq_bands: EqBandSetting[];
  enabled: boolean;
  selected_preset: string;
  selected_device: string;
  window_width: number;
  window_height: number;
};

type AudioDevice = {
  name: string;
  description: string;
};

type Effect = {
  name: string;
  key: keyof Pick<Settings, "bass_boost" | "clarity" | "ambiance" | "surround" | "dynamic_boost">;
  value: number;
  icon: string;
};

const EFFECT_KEYS: Array<{
  name: string;
  key: keyof Pick<Settings, "bass_boost" | "clarity" | "ambiance" | "surround" | "dynamic_boost">;
  icon: string;
}> = [
  { name: "Bass Boost", key: "bass_boost", icon: "M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2z" },
  { name: "Clarity", key: "clarity", icon: "M15 12a3 3 0 11-6 0 3 3 0 016 0z M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" },
  { name: "Ambiance", key: "ambiance", icon: "M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" },
  { name: "Surround Sound", key: "surround", icon: "M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" },
  { name: "Dynamic Boost", key: "dynamic_boost", icon: "M13 10V3L4 14h7v7l9-11h-7z" },
];

const EQ_BANDS = [
  { frequency: 31, label: "31" },
  { frequency: 62, label: "62" },
  { frequency: 125, label: "125" },
  { frequency: 250, label: "250" },
  { frequency: 500, label: "500" },
  { frequency: 1000, label: "1k" },
  { frequency: 2000, label: "2k" },
  { frequency: 4000, label: "4k" },
  { frequency: 8000, label: "8k" },
  { frequency: 16000, label: "16k" },
];

const PRESETS = [
  { value: "General", label: "General", desc: "Balanced sound" },
  { value: "Music", label: "Music", desc: "Enhanced for music" },
  { value: "Voice", label: "Voice", desc: "Clear vocals" },
  { value: "Streaming Video", label: "Video", desc: "Movie & streaming" },
  { value: "Bass Boost", label: "Bass Boost", desc: "Deep bass" },
];

function App() {
  const [powerEnabled, setPowerEnabled] = useState(true);
  const [effects, setEffects] = useState<Effect[]>(
    EFFECT_KEYS.map((e) => ({ ...e, value: 50 }))
  );
  const [pipewireConnected, setPipewireConnected] = useState(false);
  const [selectedPreset, setSelectedPreset] = useState("General");
  const [eqGains, setEqGains] = useState<number[]>(new Array(10).fill(0));
  const [showEq, setShowEq] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState("default");
  const [isApplying, setIsApplying] = useState(false);

  // Debounce timer for applying filter-chain changes
  const applyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Schedule apply_changes with debounce
  const scheduleApply = useCallback(() => {
    if (applyTimerRef.current) {
      clearTimeout(applyTimerRef.current);
    }
    setIsApplying(true);
    applyTimerRef.current = setTimeout(async () => {
      try {
        await invoke("apply_changes");
      } catch (error) {
        console.error("Failed to apply changes:", error);
      } finally {
        setIsApplying(false);
        applyTimerRef.current = null;
      }
    }, 600);
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const data = await invoke<Settings>("get_settings");

      setEffects(
        EFFECT_KEYS.map((e) => ({
          ...e,
          value: Math.round((data[e.key]?.value ?? 0.5) * 100),
        }))
      );

      // EQ bands: convert gain dB (-12..+12) to percentage (-100..+100)
      if (data.eq_bands && Array.isArray(data.eq_bands)) {
        setEqGains(
          data.eq_bands.map((band: EqBandSetting) =>
            Math.round(((band.gain + 12) / 24) * 200 - 100)
          )
        );
      }

      setPowerEnabled(data.enabled);
      if (data.selected_preset) {
        setSelectedPreset(data.selected_preset);
      }
      if (data.selected_device) {
        setSelectedDevice(data.selected_device);
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loadDevices = useCallback(async () => {
    try {
      const devs = await invoke<AudioDevice[]>("get_devices");
      setDevices(devs);
    } catch (error) {
      console.error("Failed to load devices:", error);
    }
  }, []);

  const checkPipewireConnection = useCallback(async () => {
    try {
      const connected = await invoke<boolean>("get_pipewire_state");
      setPipewireConnected(connected);
    } catch (error) {
      console.error("Failed to check PipeWire connection:", error);
    }
  }, []);

  useEffect(() => {
    loadSettings();
    loadDevices();
    checkPipewireConnection();
  }, [loadSettings, loadDevices, checkPipewireConnection]);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (applyTimerRef.current) {
        clearTimeout(applyTimerRef.current);
      }
    };
  }, []);

  const handleTogglePower = async () => {
    try {
      const newState = await invoke<boolean>("toggle_power");
      setPowerEnabled(newState);
    } catch (error) {
      console.error("Failed to toggle power:", error);
      setPowerEnabled((prev) => !prev);
    }
  };

  const handleEffectChange = async (index: number, value: number) => {
    const newEffects = [...effects];
    newEffects[index] = { ...newEffects[index], value };
    setEffects(newEffects);

    try {
      // Save the setting (does NOT restart filter-chain)
      await invoke("set_effect", {
        effectName: newEffects[index].name,
        value: value / 100,
      });
      // Schedule debounced apply
      scheduleApply();
    } catch (error) {
      console.error("Failed to set effect:", error);
    }
  };

  const handlePresetChange = async (preset: string) => {
    setSelectedPreset(preset);
    try {
      await invoke("load_preset", { presetName: preset });
      await loadSettings();
    } catch (error) {
      console.error("Failed to load preset:", error);
    }
  };

  const handleEqBandChange = async (index: number, value: number) => {
    const newGains = [...eqGains];
    newGains[index] = value;
    setEqGains(newGains);

    try {
      // Convert -100..100 percentage to 0..1 normalized
      const normalized = (value + 100) / 200;
      await invoke("set_eq_band", { bandIndex: index, value: normalized });
      // Schedule debounced apply
      scheduleApply();
    } catch (error) {
      console.error("Failed to set EQ band:", error);
    }
  };

  const handleDeviceChange = async (deviceName: string) => {
    setSelectedDevice(deviceName);
    try {
      await invoke("set_device", { deviceName });
    } catch (error) {
      console.error("Failed to set output device:", error);
    }
  };

  const handleResetEq = async () => {
    setEqGains(new Array(10).fill(0));
    for (let i = 0; i < 10; i++) {
      try {
        await invoke("set_eq_band", { bandIndex: i, value: 0.5 });
      } catch (error) {
        console.error("Failed to reset EQ band:", error);
      }
    }
    // Apply the reset
    scheduleApply();
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-fx-bg flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-fx-accent border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-fx-text-dim text-sm">Loading FxSonic...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-fx-bg select-none">
      {/* Header */}
      <header className="border-b border-white/5 bg-fx-bg-dark/50 backdrop-blur-sm sticky top-0 z-10">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-fx-accent to-pink-600 flex items-center justify-center shadow-lg shadow-fx-accent/20">
              <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2z" />
              </svg>
            </div>
            <div>
              <h1 className="text-xl font-bold text-white tracking-tight">FxSonic</h1>
              <p className="text-[11px] text-fx-text-dim -mt-0.5">Audio Enhancement</p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {/* Applying indicator */}
            {isApplying && (
              <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium bg-amber-500/10 text-amber-400 ring-1 ring-amber-500/20">
                <div className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
                Applying...
              </div>
            )}

            {/* PipeWire Status */}
            <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium transition-colors ${
              pipewireConnected
                ? "bg-emerald-500/10 text-emerald-400 ring-1 ring-emerald-500/20"
                : "bg-red-500/10 text-red-400 ring-1 ring-red-500/20"
            }`}>
              <div className={`w-1.5 h-1.5 rounded-full ${
                pipewireConnected ? "bg-emerald-400 animate-pulse" : "bg-red-400"
              }`} />
              {pipewireConnected ? "PipeWire" : "Disconnected"}
            </div>

            {/* Power Toggle */}
            <button
              onClick={handleTogglePower}
              className={`group relative w-12 h-12 rounded-full transition-all duration-300 flex items-center justify-center ${
                powerEnabled
                  ? "bg-fx-accent shadow-lg shadow-fx-accent/40 hover:shadow-fx-accent/60"
                  : "bg-fx-slider hover:bg-fx-slider/80"
              }`}
              title={powerEnabled ? "Disable Effects" : "Enable Effects"}
            >
              <svg
                className={`w-5 h-5 transition-all duration-300 ${
                  powerEnabled ? "text-white" : "text-fx-text-dim"
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M12 3v9m-4.5-5.5A7.5 7.5 0 1019.5 12" />
              </svg>
              {powerEnabled && (
                <div className="absolute inset-0 rounded-full bg-fx-accent/20 animate-ping" style={{ animationDuration: "2s" }} />
              )}
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-6xl mx-auto px-6 py-6">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">

          {/* Effects Panel */}
          <div className="lg:col-span-2 space-y-5">
            {/* Preset Selector (Tabs) */}
            <div className="card">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider">Preset</h2>
              </div>
              <div className="grid grid-cols-5 gap-2">
                {PRESETS.map((preset) => (
                  <button
                    key={preset.value}
                    onClick={() => handlePresetChange(preset.value)}
                    disabled={!powerEnabled}
                    className={`relative px-3 py-2.5 rounded-lg text-center transition-all duration-200 ${
                      selectedPreset === preset.value
                        ? "bg-fx-accent text-white shadow-md shadow-fx-accent/30"
                        : "bg-fx-bg hover:bg-white/5 text-fx-text-dim hover:text-fx-text"
                    } disabled:opacity-40 disabled:cursor-not-allowed`}
                  >
                    <span className="text-xs font-semibold block">{preset.label}</span>
                    <span className="text-[10px] opacity-60 block mt-0.5">{preset.desc}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Effect Sliders */}
            <div className="card">
              <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider mb-5">Effects</h2>
              <div className="space-y-5">
                {effects.map((effect, index) => (
                  <div key={effect.name} className={`group transition-opacity duration-200 ${!powerEnabled ? "opacity-40" : ""}`}>
                    <div className="flex items-center gap-3 mb-2">
                      <div className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors ${
                        powerEnabled && effect.value > 0
                          ? "bg-fx-accent/15 text-fx-accent"
                          : "bg-white/5 text-fx-text-dim"
                      }`}>
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d={effect.icon} />
                        </svg>
                      </div>
                      <label className="text-sm font-medium text-fx-text flex-1">{effect.name}</label>
                      <span className={`text-sm font-mono tabular-nums w-12 text-right ${
                        powerEnabled ? "text-fx-accent" : "text-fx-text-dim"
                      }`}>
                        {effect.value}%
                      </span>
                    </div>
                    <div className="relative pl-11">
                      <input
                        type="range"
                        min="0"
                        max="100"
                        value={effect.value}
                        onChange={(e) => handleEffectChange(index, Number(e.target.value))}
                        disabled={!powerEnabled}
                        className="w-full effect-slider"
                      />
                      {/* Fill bar */}
                      <div
                        className="absolute top-1/2 left-11 h-1.5 rounded-full -translate-y-1/2 pointer-events-none transition-all"
                        style={{
                          width: `calc(${effect.value}% * (100% - 2.75rem) / 100%)`,
                          background: powerEnabled
                            ? `linear-gradient(90deg, #e94560, #ff6b81)`
                            : "#3a3a5a",
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Side Panel */}
          <div className="space-y-5">
            {/* Output Device Card */}
            <div className="card">
              <div className="flex items-center justify-between mb-3">
                <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider">Output Device</h2>
                <button
                  onClick={loadDevices}
                  className="text-fx-text-dim hover:text-fx-accent transition-colors"
                  title="Refresh devices"
                >
                  <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </button>
              </div>
              <div className="space-y-1.5">
                {/* Default option */}
                <button
                  onClick={() => handleDeviceChange("default")}
                  className={`w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-left transition-all duration-200 ${
                    selectedDevice === "default"
                      ? "bg-fx-accent/15 ring-1 ring-fx-accent/30"
                      : "bg-fx-bg hover:bg-white/5"
                  }`}
                >
                  <div className={`w-7 h-7 rounded-lg flex items-center justify-center shrink-0 ${
                    selectedDevice === "default" ? "bg-fx-accent/20 text-fx-accent" : "bg-white/5 text-fx-text-dim"
                  }`}>
                    <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                    </svg>
                  </div>
                  <div className="min-w-0 flex-1">
                    <span className={`text-xs font-medium block truncate ${
                      selectedDevice === "default" ? "text-fx-accent" : "text-fx-text"
                    }`}>System Default</span>
                    <span className="text-[10px] text-fx-text-dim block">Auto-detect output</span>
                  </div>
                  {selectedDevice === "default" && (
                    <div className="w-1.5 h-1.5 rounded-full bg-fx-accent shrink-0" />
                  )}
                </button>

                {/* Enumerated devices */}
                {devices.map((device) => {
                  const isSelected = selectedDevice === device.name;
                  const isHeadset = device.description.toLowerCase().includes("headset") ||
                    device.description.toLowerCase().includes("headphone") ||
                    device.description.toLowerCase().includes("usb") ||
                    device.description.toLowerCase().includes("fantech");
                  const isHdmi = device.description.toLowerCase().includes("hdmi") ||
                    device.description.toLowerCase().includes("digital");
                  return (
                    <button
                      key={device.name}
                      onClick={() => handleDeviceChange(device.name)}
                      className={`w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-left transition-all duration-200 ${
                        isSelected
                          ? "bg-fx-accent/15 ring-1 ring-fx-accent/30"
                          : "bg-fx-bg hover:bg-white/5"
                      }`}
                    >
                      <div className={`w-7 h-7 rounded-lg flex items-center justify-center shrink-0 ${
                        isSelected ? "bg-fx-accent/20 text-fx-accent" : "bg-white/5 text-fx-text-dim"
                      }`}>
                        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          {isHeadset ? (
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 18h.01M8 21h8a2 2 0 002-2v-1a2 2 0 00-2-2H8a2 2 0 00-2 2v1a2 2 0 002 2zM12 15V3m-4 3a4 4 0 018 0" />
                          ) : isHdmi ? (
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                          ) : (
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                          )}
                        </svg>
                      </div>
                      <div className="min-w-0 flex-1">
                        <span className={`text-xs font-medium block truncate ${
                          isSelected ? "text-fx-accent" : "text-fx-text"
                        }`}>{device.description}</span>
                      </div>
                      {isSelected && (
                        <div className="w-1.5 h-1.5 rounded-full bg-fx-accent shrink-0" />
                      )}
                    </button>
                  );
                })}

                {devices.length === 0 && (
                  <p className="text-[10px] text-fx-text-dim text-center py-2">No devices found. Click refresh.</p>
                )}
              </div>
            </div>

            {/* Status Card */}
            <div className="card">
              <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider mb-4">Status</h2>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-fx-text-dim">Engine</span>
                  <span className={`text-sm font-medium ${powerEnabled ? "text-emerald-400" : "text-fx-text-dim"}`}>
                    {powerEnabled ? "Active" : "Bypassed"}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-fx-text-dim">Audio Backend</span>
                  <span className={`text-sm font-medium ${pipewireConnected ? "text-emerald-400" : "text-red-400"}`}>
                    {pipewireConnected ? "PipeWire" : "Not Found"}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-fx-text-dim">Preset</span>
                  <span className="text-sm font-medium text-fx-accent">{selectedPreset}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-fx-text-dim">Output</span>
                  <span className="text-sm font-medium text-fx-accent truncate max-w-[140px]">
                    {selectedDevice === "default"
                      ? "System Default"
                      : devices.find(d => d.name === selectedDevice)?.description ?? selectedDevice}
                  </span>
                </div>
              </div>
            </div>

            {/* EQ Controls */}
            <div className="card">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider">Equalizer</h2>
                <button
                  onClick={handleResetEq}
                  disabled={!powerEnabled}
                  className="text-[10px] text-fx-text-dim hover:text-fx-accent transition-colors disabled:opacity-40 font-medium uppercase tracking-wider"
                >
                  Reset
                </button>
              </div>

              {/* EQ Bars Visualization */}
              <div className="flex items-end justify-between gap-1 h-32 mb-3 px-1">
                {eqGains.map((gain, index) => {
                  const normalizedHeight = (gain + 100) / 200; // 0 to 1
                  const barHeight = Math.max(4, normalizedHeight * 100);
                  const isPositive = gain >= 0;
                  return (
                    <div key={index} className="flex-1 flex flex-col items-center gap-1 h-full justify-end">
                      <div className="relative flex-1 w-full flex items-end justify-center">
                        <div className="absolute inset-x-0 top-1/2 h-px bg-white/10" />
                        <div
                          className="w-full max-w-[14px] rounded-sm transition-all duration-150 mx-auto"
                          style={{
                            height: `${barHeight}%`,
                            background: isPositive
                              ? `linear-gradient(to top, #e94560, #ff6b81)`
                              : `linear-gradient(to bottom, #4a4a6a, #2a2a4a)`,
                            opacity: powerEnabled ? 1 : 0.3,
                          }}
                        />
                      </div>
                      <span className="text-[9px] text-fx-text-dim">{EQ_BANDS[index].label}</span>
                    </div>
                  );
                })}
              </div>

              {/* Toggle detailed EQ */}
              <button
                onClick={() => setShowEq(!showEq)}
                disabled={!powerEnabled}
                className="w-full py-2 text-xs font-medium bg-fx-bg hover:bg-white/5 text-fx-text-dim hover:text-fx-text transition-colors rounded-lg disabled:opacity-40 disabled:cursor-not-allowed"
              >
                {showEq ? "Hide Sliders" : "Show Sliders"}
              </button>

              {showEq && (
                <div className="mt-3 space-y-2 pt-3 border-t border-white/5">
                  {EQ_BANDS.map((band, index) => (
                    <div key={band.frequency} className="flex items-center gap-2">
                      <span className="w-8 text-[10px] text-fx-text-dim text-right font-mono">
                        {band.label}
                      </span>
                      <input
                        type="range"
                        min="-100"
                        max="100"
                        value={eqGains[index]}
                        onChange={(e) => handleEqBandChange(index, Number(e.target.value))}
                        disabled={!powerEnabled}
                        className="flex-1 eq-slider"
                      />
                      <span className="w-10 text-[10px] text-fx-accent font-mono text-right tabular-nums">
                        {eqGains[index] > 0 ? "+" : ""}{eqGains[index]}%
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Spectrum Placeholder */}
            <div className="card">
              <h2 className="text-sm font-semibold text-fx-text-dim uppercase tracking-wider mb-4">Spectrum</h2>
              <div className="h-24 bg-fx-bg rounded-lg flex items-center justify-center relative overflow-hidden">
                {/* Fake spectrum bars */}
                <div className="absolute inset-0 flex items-end justify-around px-2 pb-2 opacity-20">
                  {Array.from({ length: 20 }).map((_, i) => (
                    <div
                      key={i}
                      className="w-1 bg-fx-accent rounded-full"
                      style={{
                        height: `${15 + Math.sin(i * 0.8) * 40 + 30}%`,
                        opacity: powerEnabled ? 1 : 0.3,
                      }}
                    />
                  ))}
                </div>
                <span className="text-fx-text-dim/50 text-[10px] uppercase tracking-widest font-medium z-10">
                  Coming Soon
                </span>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
