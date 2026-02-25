// FxSonic DSP Test Tool
// Command-line utility to test DSP functionality

use fxsonic_dsp::{AudioProcessor, AudioParams};
use std::io::{self, Write, BufRead};

fn main() {
    println!("FxSonic DSP Test Tool");
    println!("=====================");
    println!();

    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    println!("Audio processor initialized at {} Hz", sample_rate);
    println!();

    // Test parameters
    let mut params = AudioParams::with_defaults();
    params.eq_gains = [0.0f32; 10]; // -12 to +12 dB

    let frequencies = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

    // Generate test signal
    let mut test_signal = vec![0.0f32; 480];
    for (i, sample) in test_signal.iter_mut().enumerate() {
        // Generate a 1 kHz sine wave at 20% amplitude
        *sample = 0.2 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 48000.0).sin();
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        println!("\nCurrent Parameters:");
        println!("  Enabled: {}", params.enabled);
        println!("  Bass Boost: {:.2}", params.bass_boost);
        println!("  Clarity: {:.2}", params.clarity);
        println!("  Ambiance: {:.2}", params.ambiance);
        println!("  Surround: {:.2}", params.surround);
        println!("  Dynamic Boost: {:.2}", params.dynamic_boost);
        println!();
        println!("  EQ Gains (dB):");
        for (i, freq) in frequencies.iter().enumerate() {
            println!("    {} Hz: {:.1} dB", freq, params.eq_gains[i]);
        }

        // Update processor
        processor.set_params(params);

        // Process test signal
        let mut left = test_signal.clone();
        let mut right = test_signal.clone();
        processor.process_stereo(&mut left, &mut right);

        println!("\nProcessed 480 samples of 1kHz sine wave");
        println!("  Peak input: {:.6}", test_signal.iter().map(|&x| x.abs()).reduce(f32::max).unwrap_or(0.0));
        println!("  Peak output L: {:.6}", left.iter().map(|&x| x.abs()).reduce(f32::max).unwrap_or(0.0));
        println!("  Peak output R: {:.6}", right.iter().map(|&x| x.abs()).reduce(f32::max).unwrap_or(0.0));

        println!("\nCommands:");
        println!("  b <value> - Set bass boost (0.0-1.0)");
        println!("  c <value> - Set clarity (0.0-1.0)");
        println!("  a <value> - Set ambiance (0.0-1.0)");
        println!("  s <value> - Set surround (0.0-1.0)");
        println!("  d <value> - Set dynamic boost (0.0-1.0)");
        println!("  e <band> <gain> - Set EQ band (0-9) gain in dB (-12 to +12)");
        println!("  t - Toggle enabled");
        println!("  r - Reset all parameters");
        println!("  q - Quit");

        print!("\n> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        let input = input.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "b" => {
                if parts.len() >= 2 {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        params.bass_boost = v.clamp(0.0, 1.0);
                        println!("Bass boost set to {:.2}", params.bass_boost);
                    }
                }
            }
            "c" => {
                if parts.len() >= 2 {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        params.clarity = v.clamp(0.0, 1.0);
                        println!("Clarity set to {:.2}", params.clarity);
                    }
                }
            }
            "a" => {
                if parts.len() >= 2 {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        params.ambiance = v.clamp(0.0, 1.0);
                        println!("Ambiance set to {:.2}", params.ambiance);
                    }
                }
            }
            "s" => {
                if parts.len() >= 2 {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        params.surround = v.clamp(0.0, 1.0);
                        println!("Surround set to {:.2}", params.surround);
                    }
                }
            }
            "d" => {
                if parts.len() >= 2 {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        params.dynamic_boost = v.clamp(0.0, 1.0);
                        println!("Dynamic boost set to {:.2}", params.dynamic_boost);
                    }
                }
            }
            "e" => {
                if parts.len() >= 3 {
                    if let Ok(band) = parts[1].parse::<usize>() {
                        if band < 10 {
                            if let Ok(gain) = parts[2].parse::<f32>() {
                                params.eq_gains[band] = gain.clamp(-12.0, 12.0);
                                println!("EQ band {} ({} Hz) set to {:.1} dB", band, frequencies[band], params.eq_gains[band]);
                            }
                        }
                    }
                }
            }
            "t" => {
                params.enabled = !params.enabled;
                println!("Enabled: {}", params.enabled);
            }
            "r" => {
                params = AudioParams::with_defaults();
                params.eq_gains = [0.0; 10];
                println!("All parameters reset");
            }
            "q" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
            }
        }
    }
}
