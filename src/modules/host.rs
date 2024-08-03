use core::str;
use std::path::Path;

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util, ModuleError};

pub struct HostInfo {
    host: String,
    chassis: String
}
#[derive(Deserialize)]
pub struct HostConfiguration {
    pub title: String,
    pub format: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>
}
impl Module for HostInfo {
    fn new() -> HostInfo {
        HostInfo {
            host: "".to_string(),
            chassis: "".to_string()
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
        config.host.format.replace("{host}", &self.host)
            .replace("{chassis}", &self.chassis)
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

    // Now the chassis type 
    host.chassis = match util::file_read(Path::new("/sys/devices/virtual/dmi/id/chassis_type")) {
        // http://git.savannah.nongnu.org/cgit/dmidecode.git/tree/dmidecode.c?id=d5af407ae937b0ab26b72e8c250112d5a8543a63#n602
        // I have no idea if this is meant to be hex or decimal, so I'm taking a gamble that it's
        // decimal
        Ok(r) => match r.trim() {
            "1" => "Other".to_string(),
		    "2" => "Unknown".to_string(),
		    "3" => "Desktop".to_string(),
		    "4" => "Low Profile Desktop".to_string(),
		    "5" => "Pizza Box".to_string(),
		    "6" => "Mini Tower".to_string(),
		    "7" => "Tower".to_string(),
		    "8" => "Portable".to_string(),
		    "9" => "Laptop".to_string(),
		    "10" => "Notebook".to_string(),
		    "11" => "Hand Held".to_string(),
		    "12" => "Docking Station".to_string(),
		    "13" => "All In One".to_string(),
		    "14" => "Sub Notebook".to_string(),
		    "15" => "Space-saving".to_string(),
		    "16" => "Lunch Box".to_string(),
		    "17" => "Main Server Chassis".to_string(),
		    "18" => "Expansion Chassis".to_string(),
		    "19" => "Sub Chassis".to_string(),
		    "20" => "Bus Expansion Chassis".to_string(),
		    "21" => "Peripheral Chassis".to_string(),
		    "22" => "RAID Chassis".to_string(),
		    "23" => "Rack Mount Chassis".to_string(),
		    "24" => "Sealed-case PC".to_string(),
		    "25" => "Multi-system".to_string(),
		    "26" => "CompactPCI".to_string(),
		    "27" => "AdvancedTCA".to_string(),
		    "28" => "Blade".to_string(),
		    "29" => "Blade Enclosing".to_string(),
		    "30" => "Tablet".to_string(),
		    "31" => "Convertible".to_string(),
		    "32" => "Detachable".to_string(),
		    "33" => "IoT Gateway".to_string(),
		    "34" => "Embedded PC".to_string(),
		    "35" => "Mini PC".to_string(),
		    "36" => "Stick PC".to_string(),
            _ => "Unknown".to_string()
        },
        Err(e) => return Err(ModuleError::new("Host", format!("Can't read from /sys/devices/virtual/dmi/id/chassis_type - {}", e))),
    };

    Ok(host)
}
