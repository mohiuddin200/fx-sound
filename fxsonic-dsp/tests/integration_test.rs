// Integration tests for FxSonic DSP pipeline

use fxsonic_dsp::processor::{AudioProcessor, AudioParams};

#[test]
fn test_audio_processor_initialization() {
    let sample_rate = 48000.0;
    let processor = AudioProcessor::new(sample_rate);

    // Default params should have all effects disabled
    let params = processor.get_params();
    assert_eq!(params.enabled, false);
    assert_eq!(params.bass_boost, 0.0);
    assert_eq!(params.clarity, 0.0);
    assert_eq!(params.ambiance, 0.0);
    assert_eq!(params.surround, 0.0);
    assert_eq!(params.dynamic_boost, 0.0);
}

#[test]
fn test_audio_processing_with_enabled_effects() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    // Enable all effects
    let params = AudioParams {
        enabled: true,
        bass_boost: 0.5,
        clarity: 0.5,
        ambiance: 0.5,
        surround: 0.5,
        dynamic_boost: 0.5,
        eq_gains: [0.0; 10],
    };

    processor.set_params(params);

    // Process audio
    let mut buffer_l: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();
    let mut buffer_r: Vec<f32> = buffer_l.clone();

    processor.process_stereo(&mut buffer_l, &mut buffer_r);
    // Output should be modified by effects
    assert_ne!(buffer_l, vec![0.0; 256]);
    assert_ne!(buffer_r, vec![0.0; 256]);
}

#[test]
fn test_audio_processing_disabled() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    // Keep effects disabled
    let params = AudioParams {
        enabled: false,
        bass_boost: 0.5,
        clarity: 0.5,
        ambiance: 0.5,
        surround: 0.5,
        dynamic_boost: 0.5,
        eq_gains: [0.5; 10],
    };

    processor.set_params(params);

    // Process audio - should pass through with limiter only
    let input: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();
    let mut buffer_l = input.clone();
    let mut buffer_r = input.clone();

    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // With only limiter and no effects, output should be similar to input
    // (allowing for some limiter clipping on peaks)
    let diff_l: f32 = buffer_l.iter().zip(&input).map(|(a, b)| (a - b).abs()).sum();
    let diff_r: f32 = buffer_r.iter().zip(&input).map(|(a, b)| (a - b).abs()).sum();

    // Differences should be relatively small when effects are disabled
    assert!(diff_l < input.len() as f32 * 0.1);
    assert!(diff_r < input.len() as f32 * 0.1);
}

#[test]
fn test_bass_boost_effect() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    let input: Vec<f32> = vec![0.5; 256];
    let mut buffer_l = input.clone();
    let mut buffer_r = input.clone();

    // Process without bass boost
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.0,
        clarity: 0.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [0.0; 10],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    let output_no_boost_l = buffer_l.clone();
    let output_no_boost_r = buffer_r.clone();

    // Reset buffer
    buffer_l = input.clone();
    buffer_r = input.clone();

    // Process with maximum bass boost
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 1.0,
        clarity: 0.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [0.0; 10],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // Bass boost should change the output
    assert_ne!(buffer_l, output_no_boost_l);
    assert_ne!(buffer_r, output_no_boost_r);
}

#[test]
fn test_clarity_effect() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    let mut buffer_l: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();
    let mut buffer_r: Vec<f32> = buffer_l.clone();

    // Process without clarity
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.0,
        clarity: 0.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [0.0; 10],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    let output_no_clarity_l = buffer_l.clone();

    // Reset buffer
    buffer_l = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();

    // Process with maximum clarity
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.0,
        clarity: 1.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [0.0; 10],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // Clarity should change the output
    assert_ne!(buffer_l, output_no_clarity_l);
}

#[test]
fn test_eq_bands() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    let mut buffer_l: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();
    let mut buffer_r: Vec<f32> = buffer_l.clone();

    // Process with flat EQ
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.0,
        clarity: 0.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [0.0; 10],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    let output_flat_eq_l = buffer_l.clone();

    // Reset buffer
    buffer_l = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();

    // Process with boosted EQ
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.0,
        clarity: 0.0,
        ambiance: 0.0,
        surround: 0.0,
        dynamic_boost: 0.0,
        eq_gains: [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0],
    });
    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // EQ should change the output
    assert_ne!(buffer_l, output_flat_eq_l);
}

#[test]
fn test_limiter_prevents_clipping() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    // Input with very high amplitude
    let input: Vec<f32> = vec![10.0; 256];
    let mut buffer_l = input.clone();
    let mut buffer_r = input.clone();

    // Enable all effects to potentially increase amplitude further
    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 1.0,
        clarity: 1.0,
        ambiance: 1.0,
        surround: 1.0,
        dynamic_boost: 1.0,
        eq_gains: [12.0; 10],
    });

    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // Limiter should prevent clipping beyond ±1.0
    for sample in &buffer_l {
        assert!(*sample <= 1.0 && *sample >= -1.0,
                "Limiter failed: sample {} exceeds safe range", sample);
    }
    for sample in &buffer_r {
        assert!(*sample <= 1.0 && *sample >= -1.0,
                "Limiter failed: sample {} exceeds safe range", sample);
    }
}

#[test]
fn test_multiple_processing_runs() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.5,
        clarity: 0.5,
        ambiance: 0.5,
        surround: 0.5,
        dynamic_boost: 0.5,
        eq_gains: [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
    });

    // Process multiple times in sequence
    for _ in 0..10 {
        let mut buffer_l: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) * 0.5 - 0.25).collect();
        let mut buffer_r: Vec<f32> = buffer_l.clone();

        processor.process_stereo(&mut buffer_l, &mut buffer_r);
    }
}

#[test]
fn test_variable_buffer_sizes() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.3,
        clarity: 0.3,
        ambiance: 0.3,
        surround: 0.3,
        dynamic_boost: 0.3,
        eq_gains: [0.0; 10],
    });

    // Test various buffer sizes
    let buffer_sizes = [32, 64, 128, 256, 512, 1024, 2048];

    for size in buffer_sizes {
        let mut buffer_l: Vec<f32> = (0..size).map(|i| (i as f32 / size as f32) * 0.5 - 0.25).collect();
        let mut buffer_r: Vec<f32> = buffer_l.clone();

        processor.process_stereo(&mut buffer_l, &mut buffer_r);

        // Verify output size matches input
        assert_eq!(buffer_l.len(), size);
        assert_eq!(buffer_r.len(), size);
    }
}

#[test]
fn test_silence_input() {
    let sample_rate = 48000.0;
    let mut processor = AudioProcessor::new(sample_rate);

    processor.set_params(AudioParams {
        enabled: true,
        bass_boost: 0.5,
        clarity: 0.5,
        ambiance: 0.5,
        surround: 0.5,
        dynamic_boost: 0.5,
        eq_gains: [0.0; 10],
    });

    // Silence input should remain close to silence (some noise from effects)
    let mut buffer_l: Vec<f32> = vec![0.0; 256];
    let mut buffer_r: Vec<f32> = vec![0.0; 256];

    processor.process_stereo(&mut buffer_l, &mut buffer_r);

    // Output should remain near silence (allowing for very small floating point errors)
    for sample in &buffer_l {
        assert!(sample.abs() < 0.001, "Silence input produced non-silence output: {}", sample);
    }
    for sample in &buffer_r {
        assert!(sample.abs() < 0.001, "Silence input produced non-silence output: {}", sample);
    }
}
