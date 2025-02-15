use core::str;
use std::path::Path;

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util::{self, is_flag_set_u32}, ModuleError};

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
    pub separator: Option<String>,
    pub newline_chassis: bool,
    pub chassis_title: String,
    pub chassis_format: String
}
impl Module for HostInfo {
    fn new() -> HostInfo {
        HostInfo {
            host: "Unknown".to_string(),
            chassis: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.host.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.host.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.host.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.host.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.host.title
            .replace("{host}", "Unknown")
            .replace("{chassis}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{host}", &self.host)
            .replace("{chassis}", &self.chassis)
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        // model and vendor are co-dependent
        if format.contains("{host}") {
            info_flags |= HOST_INFOFLAG_HOST;
        }
        if format.contains("{chassis}") {
            info_flags |= HOST_INFOFLAG_CHASSIS;
        }

        info_flags
    }
}
impl HostInfo {
    // Identical to the regular style method, but placeholder's in the kernel instead
    pub fn style_chassis(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.host.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.host.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.host.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.host.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&config.host.chassis_format.replace("{chassis}", &self.chassis), config);

        Self::default_style(config, &config.host.chassis_title, title_color, title_bold, title_italic, separator, &value)
    }
}

const HOST_INFOFLAG_HOST: u32 = 1;
const HOST_INFOFLAG_CHASSIS: u32 = 2;

pub fn get_host(config: &Configuration) -> Result<HostInfo, ModuleError> {
    let mut host: HostInfo = HostInfo::new();

    let mut format: String = config.host.format.to_string();
    if config.host.newline_chassis {
        format.push_str(&config.host.chassis_format);
    }
    let info_flags: u32 = HostInfo::gen_info_flags(&format);

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

    // WSL
    if util::in_wsl() {
        host.host = "Windows Subsystem for Linux".to_string();
        host.chassis = "N/A".to_string();
        return Ok(host);
    }

    // Prioritises product_name for laptops, then goes to board_name
    if is_flag_set_u32(info_flags, HOST_INFOFLAG_HOST) {
        let chosen_path: &Path = match util::find_first_path_exists(vec![
            Path::new("/sys/devices/virtual/dmi/id/product_name"),
            Path::new("/sys/devices/virtual/dmi/id/board_name"),
            Path::new("/sys/firmware/devicetree/base/model")
        ]) {
            Some(r) => r,
            None => return Err(ModuleError::new("Host", "Can't find an appropriate path for host.".to_string()))
        };

        host.host = match util::file_read(chosen_path) {
            Ok(r) => r.trim().to_string(),
            Err(e) => return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path.display(), e))),
        };
    }

    // Now the chassis type 
    if is_flag_set_u32(info_flags, HOST_INFOFLAG_CHASSIS) {
        let p: &Path = Path::new("/sys/devices/virtual/dmi/id/chassis_type");
        // The file may not exist on some stuff, e.g raspberry pi's don't have it
        if p.exists() {
            host.chassis = match util::file_read(Path::new("/sys/devices/virtual/dmi/id/chassis_type")) {
                // http://git.savannah.nongnu.org/cgit/dmidecode.git/tree/dmidecode.c?id=d5af407ae937b0ab26b72e8c250112d5a8543a63#n602
                // I have no idea if this is meant to be hex or decimal, so I'm taking a gamble that it's
                // decimal
                Ok(r) => match r.trim() {
                    "1" => "Other".to_string(),
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
                Err(e) => return Err(ModuleError::new("Host", format!("Can't read from /sys/devices/virtual/dmi/id/chassis_type - {e}"))),
            };
        }
    }

    Ok(host)
}
