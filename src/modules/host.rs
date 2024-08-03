use core::str;
use std::path::Path;

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util, ModuleError};

pub struct HostInfo {
    host: String,
}
#[derive(Deserialize)]
pub struct HostConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
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
        let separator: &str = config.host.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.host.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, separator, "Unknown")
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
        if let Some(model) = props.get("ro.product.model") {
            if let Some(manufacturer) = props.get("ro.product.manufacturer") {
                host.host = format!("{} {}", manufacturer.trim(), model.trim());
                return Ok(host);
            }
        }
    }

    // Prioritises product_name for laptops, then goes to board_name
    let chosen_path: &Path = match util::find_first_that_exists(vec![
        Path::new("/sys/devices/virtual/dmi/id/product_name"),
        Path::new("/sys/devices/virtual/dmi/id/board_name")
    ]) {
        Some(r) => r,
        None => return Err(ModuleError::new("Host", "Can't find an appropriate path for host.".to_string()))
    };

    host.host = match util::file_read(chosen_path) {
        Ok(r) => r.trim().to_string(),
        Err(e) => return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path.display(), e))),
    };

    Ok(host)
}
