// Basic tests for configuration management

#[test]
fn test_settings_default() {
    use fxsonic_app::config_manager::{Effect, Settings};

    let settings = Settings::default();
    assert_eq!(settings.bass_boost.value, 0.5);
    assert_eq!(settings.clarity.value, 0.5);
    assert_eq!(settings.ambiance.value, 0.3);
    assert_eq!(settings.surround.value, 0.4);
    assert_eq!(settings.dynamic_boost.value, 0.5);
    assert!(settings.enabled);
}

#[test]
fn test_eq_band_count() {
    use fxsonic_app::config_manager::{EqBand, Settings};

    let settings = Settings::default();
    assert_eq!(settings.eq_bands.len(), 10);
}

#[test]
fn test_serialization() {
    use fxsonic_app::config_manager::Settings;

    let settings = Settings::default();
    let serialized = toml::to_string_pretty(&settings);
    assert!(serialized.is_ok());

    let deserialized: Result<Settings, _> = toml::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
}
