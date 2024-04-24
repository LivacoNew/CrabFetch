use core::str;
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module};

pub struct OSInfo {
    distro: String,
    pub distro_id: String,
    kernel: String
}
#[derive(Deserialize)]
pub struct OSConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String
}
impl Module for OSInfo {
    fn new() -> OSInfo {
        OSInfo {
            distro: "".to_string(),
            distro_id: "".to_string(),
            kernel: "".to_string(),
        }
    }

    fn style(&self, config: &Configuration) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.os.title_color).is_some() {
            title_color = config.os.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.os.title_bold.is_some() {
            title_bold = config.os.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.os.title_italic.is_some() {
            title_italic = config.os.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.os.seperator.is_some() {
            seperator = config.os.seperator.as_ref().unwrap();
        }

        self.default_style(config, 0, &config.os.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.os.format.replace("{distro}", &self.distro)
            .replace("{kernel}", &self.kernel)
    }
}

pub fn get_os() -> OSInfo {
    let mut os: OSInfo = OSInfo::new();

    // Grabs the distro name from /etc/os-release
    // Grabs the kernel release from /proc/sys/kernel/osrelease

    // Distro
    let mut file: File = match File::open("/etc/os-release") {
        Ok(r) => r,
        Err(e) => {
            // log_error("OS", format!("Can't read from /etc/os-release - {}", e));
            return os
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            // log_error("OS", format!("Can't read from /etc/os-release - {}", e));
            return os
        },
    }
    for line in contents.trim().to_string().split("\n").collect::<Vec<&str>>() {
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
            // log_error("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e));
            return os
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            // log_error("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e));
            return os
        },
    }
    os.kernel = contents.trim().to_string();

    os
}
