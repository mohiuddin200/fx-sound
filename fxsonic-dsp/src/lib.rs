// FxSonic DSP - LADSPA Plugin
// Core audio processing engine with 5 effects + 10-band EQ

pub mod biquad;
pub mod processor;
pub mod limiter;
pub mod eq;
pub mod bass_boost;
pub mod clarity;
pub mod dynamic_boost;
pub mod surround;
pub mod ambiance;

use ladspa::{PluginDescriptor, PortDescriptor, Port, DefaultValue, Plugin};
use once_cell::sync::Lazy;

// Re-export main types for testing
pub use processor::{AudioProcessor, AudioParams};

// Port indices
pub const PORT_BASS_BOOST: usize = 0;
pub const PORT_CLARITY: usize = 1;
pub const PORT_AMBIANCE: usize = 2;
pub const PORT_SURROUND: usize = 3;
pub const PORT_DYNAMIC_BOOST: usize = 4;
pub const PORT_ENABLED: usize = 5;
pub const PORT_EQ_1: usize = 6;
pub const PORT_EQ_2: usize = 7;
pub const PORT_EQ_3: usize = 8;
pub const PORT_EQ_4: usize = 9;
pub const PORT_EQ_5: usize = 10;
pub const PORT_EQ_6: usize = 11;
pub const PORT_EQ_7: usize = 12;
pub const PORT_EQ_8: usize = 13;
pub const PORT_EQ_9: usize = 14;
pub const PORT_EQ_10: usize = 15;
pub const PORT_AUDIO_IN_L: usize = 16;
pub const PORT_AUDIO_IN_R: usize = 17;
pub const PORT_AUDIO_OUT_L: usize = 18;
pub const PORT_AUDIO_OUT_R: usize = 19;

struct FxSonicInstance {
    // Sample rate for processor
    sample_rate: f32,

    // Audio processor
    processor: AudioProcessor,
}

impl Plugin for FxSonicInstance {
    fn activate(&mut self) {}

    fn deactivate(&mut self) {}

    fn run<'a>(&mut self, sample_count: usize, ports: &[&'a ladspa::PortConnection<'a>]) {
        // Get control port values
        // Effects: 0-1 normalized
        // EQ: -1 to 1 normalized, maps to -12 to +12 dB
        let bass_boost = *ports[PORT_BASS_BOOST].unwrap_control();
        let clarity = *ports[PORT_CLARITY].unwrap_control();
        let ambiance = *ports[PORT_AMBIANCE].unwrap_control();
        let surround = *ports[PORT_SURROUND].unwrap_control();
        let dynamic_boost = *ports[PORT_DYNAMIC_BOOST].unwrap_control();
        let enabled = *ports[PORT_ENABLED].unwrap_control() > 0.5;

        let eq_gains = [
            *ports[PORT_EQ_1].unwrap_control() * 12.0,
            *ports[PORT_EQ_2].unwrap_control() * 12.0,
            *ports[PORT_EQ_3].unwrap_control() * 12.0,
            *ports[PORT_EQ_4].unwrap_control() * 12.0,
            *ports[PORT_EQ_5].unwrap_control() * 12.0,
            *ports[PORT_EQ_6].unwrap_control() * 12.0,
            *ports[PORT_EQ_7].unwrap_control() * 12.0,
            *ports[PORT_EQ_8].unwrap_control() * 12.0,
            *ports[PORT_EQ_9].unwrap_control() * 12.0,
            *ports[PORT_EQ_10].unwrap_control() * 12.0,
        ];

        // Build params struct
        let params = AudioParams {
            enabled,
            bass_boost,
            clarity,
            ambiance,
            surround,
            dynamic_boost,
            eq_gains,
        };

        // Update processor
        self.processor.set_params(params);

        // Get audio buffers
        let input_l = ports[PORT_AUDIO_IN_L].unwrap_audio();
        let input_r = ports[PORT_AUDIO_IN_R].unwrap_audio();
        let mut output_l = ports[PORT_AUDIO_OUT_L].unwrap_audio_mut();
        let mut output_r = ports[PORT_AUDIO_OUT_R].unwrap_audio_mut();

        // Copy input to output (in-place processing)
        for i in 0..sample_count {
            output_l[i] = input_l[i];
            output_r[i] = input_r[i];
        }

        // Process audio
        let output_l_slice = &mut output_l[..];
        let output_r_slice = &mut output_r[..];
        self.processor.process_stereo(output_l_slice, output_r_slice);
    }
}

// Plugin descriptor function
fn get_descriptor() -> PluginDescriptor {
    PluginDescriptor {
        unique_id: 424242,
        label: "fxsonic_enhancer",
        properties: ladspa::PROP_HARD_REALTIME_CAPABLE,
        name: "FxSonic Audio Enhancer",
        maker: "FxSonic Project",
        copyright: "MIT",
        ports: vec![
            // Control ports - Effects (normalized 0-1)
            Port {
                name: "Bass Boost",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "Clarity",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "Ambiance",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "Surround Sound",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "Dynamic Boost",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "Enabled",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: None,
                upper_bound: None,
            },
            // EQ bands (normalized -1 to +1, maps to -12 to +12 dB)
            Port {
                name: "EQ 31 Hz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 62 Hz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 125 Hz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 250 Hz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 500 Hz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 1 kHz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 2 kHz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 4 kHz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 8 kHz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            Port {
                name: "EQ 16 kHz",
                desc: PortDescriptor::ControlInput,
                hint: None,
                default: Some(DefaultValue::Value0),
                lower_bound: Some(0.0),
                upper_bound: Some(1.0),
            },
            // Audio ports
            Port {
                name: "Input L",
                desc: PortDescriptor::AudioInput,
                hint: None,
                default: None,
                lower_bound: None,
                upper_bound: None,
            },
            Port {
                name: "Input R",
                desc: PortDescriptor::AudioInput,
                hint: None,
                default: None,
                lower_bound: None,
                upper_bound: None,
            },
            Port {
                name: "Output L",
                desc: PortDescriptor::AudioOutput,
                hint: None,
                default: None,
                lower_bound: None,
                upper_bound: None,
            },
            Port {
                name: "Output R",
                desc: PortDescriptor::AudioOutput,
                hint: None,
                default: None,
                lower_bound: None,
                upper_bound: None,
            },
        ],
        new: |_: &PluginDescriptor, sample_rate: u64| {
            let sr = sample_rate as f32;
            Box::new(FxSonicInstance {
                sample_rate: sr,
                processor: AudioProcessor::new(sr),
            })
        },
    }
}

#[no_mangle]
pub extern "C" fn get_ladspa_descriptor(index: u64) -> Option<PluginDescriptor> {
    if index == 0 {
        Some(get_descriptor())
    } else {
        None
    }
}
