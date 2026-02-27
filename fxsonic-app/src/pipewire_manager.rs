// PipeWire Manager Module
// Handles PipeWire connection, device detection, and filter-chain management
//
// Architecture: The filter-chain runs as a standalone `pipewire` child process.
// When settings change, we kill the old process, write a new config, and spawn
// a new one. This avoids restarting the entire PipeWire daemon.

use std::process::{Command, Child, Stdio};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,        // internal node name (e.g. "alsa_output.pci-...")
    pub description: String, // human-readable (e.g. "Built-in Audio Analog Stereo")
}

pub struct PipeWireManager {
    connected: bool,
    config_path: String,
    filter_process: Option<Child>,
}

impl PipeWireManager {
    pub fn new() -> Result<Self, String> {
        let config_path = std::env::var("HOME")
            .unwrap_or_else(|_| ".".to_string())
            + "/.config/fxsonic/fxsonic-filter.conf";

        Ok(PipeWireManager {
            connected: false,
            config_path,
            filter_process: None,
        })
    }

    pub fn connect(&mut self) -> Result<(), String> {
        // Check if PipeWire is running
        let output = Command::new("pw-cli")
            .args(["list-objects", "Node"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                self.connected = true;
                Ok(())
            }
            _ => Err("PipeWire is not running or pw-cli is not installed".to_string()),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if the filter-chain process is still running.
    /// Returns None if no process exists, Some(true) if running, Some(false) if exited.
    pub fn is_filter_running(&mut self) -> Option<bool> {
        if let Some(ref mut child) = self.filter_process {
            match child.try_wait() {
                Ok(Some(_)) => Some(false), // Process exited
                Ok(None) => Some(true),     // Still running
                Err(_) => Some(false),      // Error checking
            }
        } else {
            None // No process started
        }
    }

    /// Start or restart the filter-chain process with current settings.
    /// The config file includes target.object for device routing, so the filter-chain
    /// will automatically route to the correct output device.
    pub fn apply_settings(&mut self, settings: &crate::config_manager::Settings) -> Result<(), String> {
        eprintln!("FxSonic: Applying settings...");

        // Stop existing filter process
        self.stop_filter();

        // Write the config file (includes target.object for device routing)
        self.write_config(settings)?;

        // Start new filter process
        self.start_filter()?;

        // Give the process time to initialize
        thread::sleep(Duration::from_millis(800));

        // Wait for the node to appear with retries, then set as default sink
        let mut found = false;
        for attempt in 0..20 {
            // Check if the filter process is still alive
            if let Some(ref mut child) = self.filter_process {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return Err(format!("Filter-chain process exited unexpectedly with status: {}. Check ~/.config/fxsonic/filter-chain.log for details.", status));
                    }
                    Ok(None) => {} // Still running
                    Err(e) => {
                        eprintln!("Warning: Error checking process status: {}", e);
                    }
                }
            }

            // Try to set FxSonic as default sink
            match self.set_fxsonic_as_default() {
                Ok(_) => {
                    found = true;
                    eprintln!("FxSonic: Successfully set as default sink");
                    break;
                }
                Err(e) => {
                    if attempt < 5 {
                        eprintln!("FxSonic: Waiting for node to appear (attempt {}/20): {}", attempt + 1, e);
                    }
                }
            }

            // Exponential backoff for retries
            let delay = if attempt < 3 { 300 } else if attempt < 10 { 500 } else { 800 };
            thread::sleep(Duration::from_millis(delay));
        }

        if !found {
            eprintln!("FxSonic: Warning - Could not set as default sink after 20 attempts");
            eprintln!("FxSonic: The filter-chain may still be running. Check status with: wpctl status");
            return Err("Failed to set FxSonic as default sink after multiple attempts. The audio may not route through FxSonic.".to_string());
        }

        eprintln!("FxSonic: Settings applied successfully");
        Ok(())
    }

    /// Stop the filter-chain child process.
    fn stop_filter(&mut self) {
        // Kill the stored child process
        if let Some(ref mut child) = self.filter_process {
            let pid = child.id();
            let _ = child.kill();
            let _ = child.wait();
            eprintln!("FxSonic: Stopped filter-chain process (PID {})", pid);
        }
        self.filter_process = None;

        // Brief pause to let PipeWire clean up the old nodes
        thread::sleep(Duration::from_millis(400));
    }

    /// Start the filter-chain as a standalone PipeWire process.
    fn start_filter(&mut self) -> Result<(), String> {
        let log_path = std::env::var("HOME")
            .unwrap_or_else(|_| ".".to_string())
            + "/.config/fxsonic/filter-chain.log";

        // Clear or create log file
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        let child = Command::new("pipewire")
            .args(["-c", &self.config_path])
            .stdout(Stdio::from(log_file.try_clone().unwrap()))
            .stderr(Stdio::from(log_file))
            .spawn()
            .map_err(|e| format!("Failed to start filter-chain process: {}. Is pipewire installed?", e))?;

        eprintln!("FxSonic: Started filter-chain process (PID {})", child.id());
        eprintln!("FxSonic: Config file: {}", self.config_path);
        eprintln!("FxSonic: Log file: {}", log_path);
        self.filter_process = Some(child);
        Ok(())
    }

    /// Find the WirePlumber ID of the fxsonic_sink node.
    fn find_fxsonic_wpctl_id(&self) -> Result<u32, String> {
        let output = Command::new("wpctl")
            .args(["status"])
            .output()
            .map_err(|e| format!("Failed to run wpctl: {}", e))?;

        let status = String::from_utf8_lossy(&output.stdout);

        // Look for FxSonic in the Sinks section
        // Format: "     35. FxSonic Audio Enhancer              [vol: 1.00]"
        // or:     " *   35. FxSonic Audio Enhancer              [vol: 1.00]"
        for line in status.lines() {
            if line.contains("FxSonic") && !line.contains("fxsonic_playback") {
                let trimmed = line.trim();
                // Extract the number before the dot
                if let Some(dot_pos) = trimmed.find('.') {
                    let id_str = trimmed[..dot_pos].trim().trim_start_matches('*').trim();
                    if let Ok(id) = id_str.parse::<u32>() {
                        return Ok(id);
                    }
                }
            }
        }

        Err("FxSonic sink not found in wpctl status. The filter-chain process may not be running yet.".to_string())
    }

    /// Set FxSonic as the default audio sink so all apps route through it.
    fn set_fxsonic_as_default(&self) -> Result<(), String> {
        let id = self.find_fxsonic_wpctl_id()?;

        let output = Command::new("wpctl")
            .args(["set-default", &id.to_string()])
            .output()
            .map_err(|e| format!("Failed to execute wpctl set-default: {}", e))?;

        if !output.status.success() {
            return Err(format!("wpctl set-default failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }

        eprintln!("FxSonic: Set as default sink (ID {})", id);
        Ok(())
    }

    /// Route fxsonic_playback output to a specific physical audio sink.
    pub fn route_to_device(&self, device_name: &str) -> Result<(), String> {
        // Disconnect existing links from fxsonic_playback
        let _ = Command::new("pw-link")
            .args(["-d", "fxsonic_playback:output_FL"])
            .output();
        let _ = Command::new("pw-link")
            .args(["-d", "fxsonic_playback:output_FR"])
            .output();

        // Connect to the new device
        let fl_target = format!("{}:playback_FL", device_name);
        let fr_target = format!("{}:playback_FR", device_name);

        let fl_result = Command::new("pw-link")
            .args(["fxsonic_playback:output_FL", &fl_target])
            .output()
            .map_err(|e| format!("Failed to link FL: {}", e))?;

        let fr_result = Command::new("pw-link")
            .args(["fxsonic_playback:output_FR", &fr_target])
            .output()
            .map_err(|e| format!("Failed to link FR: {}", e))?;

        if !fl_result.status.success() {
            return Err(format!("Failed to link FL to {}: {}",
                fl_target, String::from_utf8_lossy(&fl_result.stderr)));
        }
        if !fr_result.status.success() {
            return Err(format!("Failed to link FR to {}: {}",
                fr_target, String::from_utf8_lossy(&fr_result.stderr)));
        }

        eprintln!("Routed fxsonic_playback to {}", device_name);
        Ok(())
    }

    /// Enumerate all PipeWire audio output sinks (excluding the FxSonic virtual sink).
    pub fn get_audio_sinks(&self) -> Result<Vec<AudioDevice>, String> {
        let output = Command::new("pw-dump")
            .output()
            .map_err(|e| format!("Failed to run pw-dump: {}", e))?;

        if !output.status.success() {
            return Err("pw-dump returned non-zero exit code".to_string());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let objects: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse pw-dump JSON: {}", e))?;

        let mut devices = Vec::new();

        for obj in &objects {
            let obj_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if obj_type != "PipeWire:Interface:Node" {
                continue;
            }

            let props = obj
                .pointer("/info/props")
                .unwrap_or(&serde_json::Value::Null);

            let media_class = props
                .get("media.class")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if media_class != "Audio/Sink" {
                continue;
            }

            let node_name = props
                .get("node.name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Skip the FxSonic virtual sink itself
            if node_name == "fxsonic_sink" {
                continue;
            }

            let description = props
                .get("node.description")
                .and_then(|v| v.as_str())
                .unwrap_or(node_name);

            devices.push(AudioDevice {
                name: node_name.to_string(),
                description: description.to_string(),
            });
        }

        Ok(devices)
    }

    /// Generate the standalone PipeWire config file for the filter-chain.
// FIXED: Use simpler module-style config that works

pub fn generate_filter_chain_config(&self, settings: &crate::config_manager::Settings) -> Result<String, String> {
    let config = format!(r#"# FxSonic PipeWire Filter-Chain Configuration
# This file is automatically generated by FxSonic

context.modules = [
    {{   name = libpipewire-module-filter-chain
        args = {{
            node.description = "FxSonic Audio Enhancer"
            media.name       = "FxSonic"
            filter.graph = {{
                nodes = [
                    {{
                        type   = ladspa
                        name   = fxsonic
                        plugin = "/home/mohiuddin/.ladspa/libfxsonic_dsp.so"
                        label  = fxsonic_enhancer
                        control = {{
                            "Bass Boost"     = {}
                            "Clarity"        = {}
                            "Ambiance"       = {}
                            "Surround Sound" = {}
                            "Dynamic Boost"  = {}
                            "EQ 31 Hz"       = {}
                            "EQ 62 Hz"       = {}
                            "EQ 125 Hz"      = {}
                            "EQ 250 Hz"      = {}
                            "EQ 500 Hz"      = {}
                            "EQ 1 kHz"       = {}
                            "EQ 2 kHz"       = {}
                            "EQ 4 kHz"       = {}
                            "EQ 8 kHz"       = {}
                            "EQ 16 kHz"      = {}
                            "Enabled"        = {}
                        }}
                    }}
                ]
            }}
            capture.props = {{
                node.name      = "fxsonic_sink"
                media.class    = Audio/Sink
                audio.position = [ FL FR ]
            }}
            playback.props = {{
                node.name      = "fxsonic_playback"
                node.passive   = true
                audio.position = [ FL FR ]
            }}
        }}
    }}
]
"#,
            settings.bass_boost.value,
            settings.clarity.value,
            settings.ambiance.value,
            settings.surround.value,
            settings.dynamic_boost.value,
            (settings.eq_bands[0].gain + 12.0) / 24.0,
            (settings.eq_bands[1].gain + 12.0) / 24.0,
            (settings.eq_bands[2].gain + 12.0) / 24.0,
            (settings.eq_bands[3].gain + 12.0) / 24.0,
            (settings.eq_bands[4].gain + 12.0) / 24.0,
            (settings.eq_bands[5].gain + 12.0) / 24.0,
            (settings.eq_bands[6].gain + 12.0) / 24.0,
            (settings.eq_bands[7].gain + 12.0) / 24.0,
            (settings.eq_bands[8].gain + 12.0) / 24.0,
            (settings.eq_bands[9].gain + 12.0) / 24.0,
            if settings.enabled { 1.0 } else { 0.0 }
        );
        Ok(config)
    }
    pub fn write_config(&self, settings: &crate::config_manager::Settings) -> Result<(), String> {
        let config_content = self.generate_filter_chain_config(settings)?;

        std::fs::create_dir_all(
            std::path::Path::new(&self.config_path).parent()
                .ok_or("Invalid config path")?
        ).map_err(|e| format!("Failed to create config directory: {}", e))?;

        std::fs::write(&self.config_path, config_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Clean shutdown: stop the filter process.
    pub fn shutdown(&mut self) {
        self.stop_filter();
    }
}

impl Default for PipeWireManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PipeWireManager")
    }
}

impl Drop for PipeWireManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
