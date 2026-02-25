// Configuration Manager Module
// Handles loading, saving, and managing user configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub frequency: f64,
    pub gain: f64,
    pub q: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub enabled: bool,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub bass_boost: Effect,
    pub clarity: Effect,
    pub ambiance: Effect,
    pub surround: Effect,
    pub dynamic_boost: Effect,
    pub eq_bands: [EqBand; 10],
    pub enabled: bool,
    pub selected_preset: String,
    pub selected_device: String,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            bass_boost: Effect { enabled: true, value: 0.5 },
            clarity: Effect { enabled: true, value: 0.5 },
            ambiance: Effect { enabled: true, value: 0.3 },
            surround: Effect { enabled: true, value: 0.4 },
            dynamic_boost: Effect { enabled: true, value: 0.5 },
            eq_bands: [
                EqBand { frequency: 31.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 62.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 125.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 250.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 500.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 1000.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 2000.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 4000.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 8000.0, gain: 0.0, q: 1.0 },
                EqBand { frequency: 16000.0, gain: 0.0, q: 1.0 },
            ],
            enabled: true,
            selected_preset: "General".to_string(),
            selected_device: "default".to_string(),
            window_width: 900,
            window_height: 700,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub settings: Settings,
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
    presets_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, String> {
        let project_dirs = ProjectDirs::from("com", "fxsonic", "fxsonic")
            .ok_or_else(|| "Failed to get project directories".to_string())?;

        let config_dir = project_dirs.config_dir().to_path_buf();
        let config_file = config_dir.join("config.toml");
        let presets_dir = config_dir.join("presets");

        // Create directories if they don't exist
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        fs::create_dir_all(&presets_dir)
            .map_err(|e| format!("Failed to create presets directory: {}", e))?;

        Ok(ConfigManager {
            config_dir,
            config_file,
            presets_dir,
        })
    }

    pub fn load_settings(&self) -> Result<Settings, String> {
        if !self.config_file.exists() {
            // Return default settings if config doesn't exist
            return Ok(Settings::default());
        }

        let contents = fs::read_to_string(&self.config_file)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let settings: Settings = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let contents = toml::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.config_file, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn load_preset(&self, name: &str) -> Result<Preset, String> {
        let preset_file = self.presets_dir.join(format!("{}.toml", name.to_lowercase()));

        let contents = fs::read_to_string(&preset_file)
            .map_err(|e| format!("Failed to read preset file: {}", e))?;

        let preset: Preset = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse preset file: {}", e))?;

        Ok(preset)
    }

    pub fn save_preset(&self, preset: &Preset) -> Result<(), String> {
        let preset_file = self.presets_dir.join(format!("{}.toml", preset.name.to_lowercase()));

        let contents = toml::to_string_pretty(preset)
            .map_err(|e| format!("Failed to serialize preset: {}", e))?;

        fs::write(&preset_file, contents)
            .map_err(|e| format!("Failed to write preset file: {}", e))?;

        Ok(())
    }

    pub fn initialize_default_presets(&self) -> Result<(), String> {
        // Create default presets
        let presets = vec![
            Preset {
                name: "General".to_string(),
                settings: Settings::default(),
            },
            Preset {
                name: "Music".to_string(),
                settings: {
                    let mut s = Settings::default();
                    s.bass_boost.value = 0.6;
                    s.clarity.value = 0.5;
                    s.ambiance.value = 0.2;
                    s.surround.value = 0.3;
                    s.dynamic_boost.value = 0.5;
                    s.selected_preset = "Music".to_string();
                    s
                },
            },
            Preset {
                name: "Voice".to_string(),
                settings: {
                    let mut s = Settings::default();
                    s.bass_boost.value = 0.2;
                    s.clarity.value = 0.8;
                    s.ambiance.value = 0.1;
                    s.surround.value = 0.2;
                    s.dynamic_boost.value = 0.7;
                    s.selected_preset = "Voice".to_string();
                    s
                },
            },
            Preset {
                name: "Streaming Video".to_string(),
                settings: {
                    let mut s = Settings::default();
                    s.bass_boost.value = 0.4;
                    s.clarity.value = 0.6;
                    s.ambiance.value = 0.3;
                    s.surround.value = 0.5;
                    s.dynamic_boost.value = 0.6;
                    s.selected_preset = "Streaming Video".to_string();
                    s
                },
            },
            Preset {
                name: "Bass Boost".to_string(),
                settings: {
                    let mut s = Settings::default();
                    s.bass_boost.value = 0.9;
                    s.clarity.value = 0.3;
                    s.ambiance.value = 0.2;
                    s.surround.value = 0.3;
                    s.dynamic_boost.value = 0.7;
                    s.selected_preset = "Bass Boost".to_string();
                    s
                },
            },
        ];

        for preset in presets {
            self.save_preset(&preset)?;
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigManager")
    }
}
