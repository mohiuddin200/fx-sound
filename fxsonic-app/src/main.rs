// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::State;

use fxsonic_app::config_manager::{ConfigManager, Settings};
use fxsonic_app::pipewire_manager::{AudioDevice, PipeWireManager};

// Application state
struct AppState {
    pipewire_manager: Mutex<PipeWireManager>,
    config_manager: Mutex<ConfigManager>,
    settings: Mutex<Settings>,
}

// PipeWire state check
#[tauri::command]
fn get_pipewire_state(state: State<AppState>) -> Result<bool, String> {
    let manager = state.pipewire_manager.lock().unwrap();
    Ok(manager.is_connected())
}

// Toggle power - restarts filter-chain with new enabled state
#[tauri::command]
fn toggle_power(state: State<AppState>) -> Result<bool, String> {
    let mut settings = state.settings.lock().unwrap();
    settings.enabled = !settings.enabled;

    // Save config
    let config = state.config_manager.lock().unwrap();
    config.save_settings(&*settings).map_err(|e: String| e.to_string())?;

    // Restart filter-chain with new settings
    let mut manager = state.pipewire_manager.lock().unwrap();
    manager.apply_settings(&*settings)?;

    Ok(settings.enabled)
}

// Apply current settings (restart filter-chain)
// Called from frontend after debounced changes
#[tauri::command]
fn apply_changes(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap();
    let mut manager = state.pipewire_manager.lock().unwrap();
    manager.apply_settings(&*settings)?;
    Ok(())
}

// Set effect parameter (saves to config but does NOT restart filter-chain)
// The frontend will call apply_changes after debouncing
#[tauri::command]
fn set_effect(effect_name: String, value: f64, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();

    // Map effect name to settings field
    match effect_name.as_str() {
        "Bass Boost" => {
            settings.bass_boost.value = value;
            settings.bass_boost.enabled = value > 0.0;
        }
        "Clarity" => {
            settings.clarity.value = value;
            settings.clarity.enabled = value > 0.0;
        }
        "Ambiance" => {
            settings.ambiance.value = value;
            settings.ambiance.enabled = value > 0.0;
        }
        "Surround Sound" => {
            settings.surround.value = value;
            settings.surround.enabled = value > 0.0;
        }
        "Dynamic Boost" => {
            settings.dynamic_boost.value = value;
            settings.dynamic_boost.enabled = value > 0.0;
        }
        _ => return Err(format!("Unknown effect: {}", effect_name)),
    }

    // Save settings
    let config = state.config_manager.lock().unwrap();
    config.save_settings(&*settings).map_err(|e: String| e.to_string())?;

    Ok(())
}

// Set EQ band (saves but does NOT restart; frontend calls apply_changes)
#[tauri::command]
fn set_eq_band(band_index: usize, value: f64, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();

    if band_index >= 10 {
        return Err("Invalid EQ band index".to_string());
    }

    // Map 0-1 to -12 to +12 dB
    let gain_db = (value * 24.0) - 12.0;
    settings.eq_bands[band_index].gain = gain_db;

    // Save settings
    let config = state.config_manager.lock().unwrap();
    config.save_settings(&*settings).map_err(|e: String| e.to_string())?;

    Ok(())
}

// Load preset - applies immediately (restarts filter-chain)
#[tauri::command]
fn load_preset(preset_name: String, state: State<AppState>) -> Result<(), String> {
    let config = state.config_manager.lock().unwrap();
    let preset = config.load_preset(&preset_name).map_err(|e: String| e.to_string())?;

    let mut settings = state.settings.lock().unwrap();
    *settings = preset.settings;

    // Save the settings
    config.save_settings(&*settings).map_err(|e: String| e.to_string())?;
    drop(config);

    // Restart filter-chain with new settings
    let mut manager = state.pipewire_manager.lock().unwrap();
    manager.apply_settings(&*settings)?;

    Ok(())
}

// Get available audio output devices from PipeWire
#[tauri::command]
fn get_devices(state: State<AppState>) -> Result<Vec<AudioDevice>, String> {
    let manager = state.pipewire_manager.lock().unwrap();
    manager.get_audio_sinks()
}

// Set the target output device - restarts filter-chain with new target.object
#[tauri::command]
fn set_device(device_name: String, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    settings.selected_device = device_name;

    // Save settings
    let config = state.config_manager.lock().unwrap();
    config.save_settings(&*settings).map_err(|e: String| e.to_string())?;
    drop(config);

    // Restart filter-chain with new target.object in config
    let mut manager = state.pipewire_manager.lock().unwrap();
    manager.apply_settings(&*settings)?;

    Ok(())
}

// Get current settings
#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<serde_json::Value, String> {
    let settings = state.settings.lock().unwrap();
    Ok(serde_json::to_value(&*settings).map_err(|e| e.to_string())?)
}

fn main() {
    // Remove any old conf.d filter-chain config to avoid conflicts
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let old_conf = format!("{}/.config/pipewire/pipewire.conf.d/fxsonic-filter.conf", home);
    let _ = std::fs::remove_file(&old_conf);

    // Initialize managers
    let mut pipewire_manager: PipeWireManager = match PipeWireManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to create PipeWire manager: {}", e);
            return;
        }
    };

    let config_manager: ConfigManager = match ConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to create config manager: {}", e);
            return;
        }
    };

    // Load settings from file
    let settings: Settings = match config_manager.load_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load settings, using defaults: {}", e);
            Settings::default()
        }
    };

    // Initialize default presets if needed
    let _ = config_manager.initialize_default_presets();

    // Connect to PipeWire
    let _ = pipewire_manager.connect();

    // Start the filter-chain process with current settings
    if let Err(e) = pipewire_manager.apply_settings(&settings) {
        eprintln!("Failed to start filter-chain: {}", e);
    }

    let app_state = AppState {
        pipewire_manager: Mutex::new(pipewire_manager),
        config_manager: Mutex::new(config_manager),
        settings: Mutex::new(settings),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_pipewire_state,
            toggle_power,
            set_effect,
            set_eq_band,
            load_preset,
            get_devices,
            set_device,
            get_settings,
            apply_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
