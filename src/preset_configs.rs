use config::{builder::DefaultState, Config, ConfigBuilder, FileFormat};

use crate::config_manager::Configuration;

const BASIC_PRESET: &str = include_str!("../presets/basic.toml");
const NEOFETCH_PRESET: &str = include_str!("../presets/neofetch.toml");
const FULL_PRESET: &str = include_str!("../presets/full.toml");

pub fn preset_full() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = builder.add_source(config::File::from_str(FULL_PRESET, FileFormat::Toml).required(true));

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}

pub fn preset_neofetch() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = builder.add_source(config::File::from_str(NEOFETCH_PRESET, FileFormat::Toml).required(true));

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}

pub fn preset_basic() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = builder.add_source(config::File::from_str(BASIC_PRESET, FileFormat::Toml).required(true));

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}
