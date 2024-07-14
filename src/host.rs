use core::str;
use std::{env, fs::File, io::Read, path::Path};

#[cfg(feature = "android")]
use android_system_properties::AndroidSystemProperties;
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

pub struct HostInfo {
    host: String,
}
#[derive(Deserialize)]
pub struct HostConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: Option<String>
}
impl Module for HostInfo {
    fn new() -> HostInfo {
        HostInfo {
            host: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.host.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.host.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let format: String = config.host.format.clone().unwrap_or("{host}".to_string());
        format.replace("{host}", &self.host)
    }
}

pub fn get_host() -> Result<HostInfo, ModuleError> {
    let mut host: HostInfo = HostInfo::new();

    // Android 
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        // Use the system property instead
        // Using some random ass crate for this, as I'm too dumb to bind to the actual C methods
        // from rust
        // If you want to figure it out yourself and help me out in a PR it's this I need;
        // https://android.googlesource.com/platform/system/core/+/refs/heads/android12-dev/libcutils/properties.cpp#85
        let props = AndroidSystemProperties::new();
        // https://github.com/termux/termux-api/issues/448#issuecomment-927345222
        if let Some(val) = props.get("ro.product.model") {
            host.host = val.trim().to_string();
            return Ok(host);
        }
    }

    // Prioritises product_name for laptops, then goes to board_name
    let mut chosen_path: Option<&str> = None;
    if Path::new("/sys/devices/virtual/dmi/id/product_name").exists() {
        chosen_path = Some("/sys/devices/virtual/dmi/id/product_name");
    } else if Path::new("/sys/devices/virtual/dmi/id/board_name").exists() {
        chosen_path = Some("/sys/devices/virtual/dmi/id/board_name");
    }
    if chosen_path.is_none() {
        return Err(ModuleError::new("Host", "Can't find an appropriate path for host.".to_string()));
    }
    let chosen_path: &str = chosen_path.unwrap();

    let mut file: File = match File::open(chosen_path) {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path, e)));
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path, e)));
        },
    }

    host.host = contents.trim().to_string();

    Ok(host)
}
