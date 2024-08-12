use core::str;
use std::path::Path;

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::{process::Command, env}};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util, ModuleError};

pub struct OSInfo {
    distro: String,
    pub distro_id: String,
    kernel: String,
}
#[derive(Deserialize)]
pub struct OSConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub newline_kernel: bool,
    pub kernel_title: String,
    pub kernel_format: String
}
impl Module for OSInfo {
    fn new() -> OSInfo {
        OSInfo {
            distro: "Unknown".to_string(),
            distro_id: "Unknown".to_string(),
            kernel: "Unknown".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.os.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.os.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.os.format.replace("{distro}", &self.distro)
            .replace("{kernel}", &self.kernel)
    }

    fn gen_info_flags(format: &str) -> u32 {
        todo!()
    }
}
impl OSInfo {
    // Identical to the regular style method, but placeholder's in the kernel instead
    pub fn style_kernel(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&config.os.kernel_format.replace("{kernel}", &self.kernel));

        Self::default_style(config, max_title_size, &config.os.kernel_title, title_color, title_bold, title_italic, separator, &value)
    }
}

pub fn get_os() -> Result<OSInfo, ModuleError> {
    let mut os: OSInfo = OSInfo::new();

    // Android 
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        let props = AndroidSystemProperties::new();
        // https://github.com/termux/termux-api/issues/448#issuecomment-927345222
        if let Some(val) = props.get("ro.build.version.release_or_preview_display") {
            os.distro = format!("Android {}", val);
        } else {
            os.distro = "Android".to_string();
        }
        os.distro_id = "android".to_string();
        
        // This may fuck performance, will have to keep an eye on this 
        let output: Vec<u8> = match Command::new("uname").arg("-r")
            .output() {
                Ok(r) => r.stdout,
                Err(_) => return Err(ModuleError::new("OS", "Can't find kernel version.".to_string())),
            };

        os.kernel = match String::from_utf8(output) {
            Ok(r) => r.trim().to_string(),
            Err(_) => return Err(ModuleError::new("OS", "Can't find kernel version.".to_string())),
        };

        return Ok(os);
    }

    // Grabs the distro name from /etc/os-release
    // Grabs the kernel release from /proc/sys/kernel/osrelease

    // Distro
    let contents = match util::file_read(Path::new("/etc/os-release")) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("OS", format!("Can't read from /etc/os-release - {}", e))),
    };
    for line in contents.trim().to_string().split('\n').collect::<Vec<&str>>() {
        if line.starts_with("PRETTY_NAME=") {
            os.distro = line[13..line.len() - 1].to_string();
            continue;
        }
        if line.starts_with("ID=") {
            os.distro_id = line[3..line.len()].trim().to_string();
            continue;
        }
    }

    // Kernel
    let contents = match util::file_read(Path::new("/proc/sys/kernel/osrelease")) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e))),
    };
    os.kernel = contents.trim().to_string();

    Ok(os)
}
