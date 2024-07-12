use core::str;
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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
    pub seperator: Option<String>,
    pub format: String,
    pub newline_kernel: bool,
    pub kernel_title: String,
    pub kernel_format: String
}
impl Module for OSInfo {
    fn new() -> OSInfo {
        OSInfo {
            distro: "".to_string(),
            distro_id: "".to_string(),
            kernel: "".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.os.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.os.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.os.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.os.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.os.format.replace("{distro}", &self.distro)
            .replace("{kernel}", &self.kernel)
    }
}
impl OSInfo {
    // Identical to the regular style method, but placeholder's in the kernel instead
    pub fn style_kernel(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.os.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&config.os.kernel_format.replace("{kernel}", &self.kernel));

        Self::default_style(config, max_title_size, &config.os.kernel_title, title_color, title_bold, title_italic, seperator, &value)
    }
}

pub fn get_os() -> Result<OSInfo, ModuleError> {
    let mut os: OSInfo = OSInfo::new();

    // Grabs the distro name from /etc/os-release
    // Grabs the kernel release from /proc/sys/kernel/osrelease

    // Distro
    let mut file: File = match File::open("/etc/os-release") {
        Ok(r) => r,
        Err(e) => {
            // log_error("OS", format!("Can't read from /etc/os-release - {}", e));
            return Err(ModuleError::new("OS", format!("Can't read from /etc/os-release - {}", e)))
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("OS", format!("Can't read from /etc/os-release - {}", e)));
        },
    }
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
    let mut file: File = match File::open("/proc/sys/kernel/osrelease") {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e)));
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e)));
        },
    }
    os.kernel = contents.trim().to_string();

    Ok(os)
}
