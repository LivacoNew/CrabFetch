use config::{builder::DefaultState, Config, ConfigBuilder};

use crate::config_manager::{fill_builder_defaults, Configuration};

pub fn preset_full() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = fill_builder_defaults(builder);
    builder = builder.set_override("modules", vec![
        "hostname".to_string(),
        "underline:16".to_string(),
        "cpu".to_string(),
        "gpu".to_string(),
        "memory".to_string(),
        "swap".to_string(),
        "mounts".to_string(),
        "host".to_string(),
        "displays".to_string(),
        "os".to_string(),
        "packages".to_string(),
        "desktop".to_string(),
        "terminal".to_string(),
        "shell".to_string(),
        "battery".to_string(),
        "uptime".to_string(),
        "locale".to_string(),
        "player".to_string(),
        "editor".to_string(),
        "initsys".to_string(),
        "processes".to_string(),
        "datetime".to_string(),
        "localip".to_string(),
        "theme".to_string(),
        "icontheme".to_string(),
        "colors".to_string(),
        "bright_colors".to_string()
    ]).expect("Failed to override default config.");

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}

pub fn preset_neofetch() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = fill_builder_defaults(builder);
    builder = builder.set_override("modules", vec![
        "hostname".to_string(),
        "underline:16".to_string(),
        "os".to_string(),
        "host".to_string(),
        "uptime".to_string(),
        "packages".to_string(),
        "shell".to_string(),
        "displays".to_string(),
        "desktop".to_string(),
        "theme".to_string(),
        "icontheme".to_string(),
        "terminal".to_string(),
        "cpu".to_string(),
        "gpu".to_string(),
        "memory".to_string(),
        "mounts".to_string(),
        #[cfg(feature = "player")]
        "player".to_string(),
        "localip".to_string(),
        "locale".to_string(),
        "space".to_string(),
        "colors".to_string(),
        "bright_colors".to_string()
    ]).expect("Failed to override default config.");
    builder = builder.set_override("underline_character", "-").expect("Failed to override default config.");
    builder = builder.set_override("separator", ": ").expect("Failed to override default config.");
    builder = builder.set_override::<&str, Vec<&str>>("percentage_color_thresholds", vec![]).expect("Failed to override default config.");
    builder = builder.set_override("decimal_places", 0).expect("Failed to override default config.");
    builder = builder.set_override("os.title", "OS").expect("Failed to override default config.");
    builder = builder.set_override("os.format", "{distro}").expect("Failed to override default config.");
    builder = builder.set_override("os.newline_kernel", true).expect("Failed to override default config.");
    builder = builder.set_override("host.format", "{host}").expect("Failed to override default config.");
    builder = builder.set_override("theme.format", "{gtk2} [GTK2], {gtk3} [GTK3]").expect("Failed to override default config.");
    builder = builder.set_override("icontheme.format", "{gtk2} [GTK2], {gtk3} [GTK3]").expect("Failed to override default config.");
    builder = builder.set_override("cpu.format", "{name} ({thread_count}) @ {max_clock_ghz}GHz").expect("Failed to override default config.");
    builder = builder.set_override("cpu.decimal_places", 3).expect("Failed to override default config.");
    builder = builder.set_override("gpu.format", "{vendor} {model}").expect("Failed to override default config.");
    builder = builder.set_override("memory.format", "{used} / {max}").expect("Failed to override default config.");
    builder = builder.set_override("mounts.format", "{space_used} / {space_total} ({percent})").expect("Failed to override default config.");
    builder = builder.set_override("locale.format", "{language}.{encoding}").expect("Failed to override default config.");

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}

pub fn preset_basic() -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = fill_builder_defaults(builder);
    builder = builder.set_override("modules", vec![
        "{color-white}┌─────────────────────────────────────{color-title} Software {color-white}─────────────────────────────────────┐".to_string(),
        "os".to_string(),
        "packages".to_string(),
        "shell".to_string(),
        "terminal".to_string(),
        "uptime".to_string(),
        "segment:Hardware".to_string(),
        "cpu".to_string(),
        "gpu".to_string(),
        "memory".to_string(),
        "host".to_string(),
        "displays".to_string(),
        "{color-white}└────────────────────────────────────────────────────────────────────────────────────┘".to_string()
    ]).expect("Failed to override default config.");
    builder = builder.set_override("unknown_as_text", true).expect("Failed to override default config.");
    builder = builder.set_override("separator", "  ").expect("Failed to override default config.");
    builder = builder.set_override("segment_top", "{color-white}├─────────────────────────────────────{color-title} {name} {color-white}─────────────────────────────────────┤").expect("Failed to override default config.");


    builder = builder.set_override("cpu.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("cpu.format", "{name} ({core_count}c/{thread_count}t) @ {max_clock_ghz} GHz").expect("Failed to override default config.");

    builder = builder.set_override("gpu.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("gpu.format", "{model} ({vram})").expect("Failed to override default config.");

    builder = builder.set_override("memory.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("host.title", "  󰍹").expect("Failed to override default config.");
    builder = builder.set_override("displays.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("os.title", "  󰘳").expect("Failed to override default config.");
    builder = builder.set_override("packages.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("terminal.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("shell.title", "  ").expect("Failed to override default config.");
    builder = builder.set_override("uptime.title", "  ").expect("Failed to override default config.");

    let config: Config = builder.build().expect("Failed to build configuration.");
    let deserialized: Configuration = config.try_deserialize::<Configuration>().expect("Failed to deserialize configuration.");
    deserialized
}
