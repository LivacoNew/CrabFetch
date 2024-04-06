use core::str;
use std::{fmt::Display, fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.os.title_color).is_some() {
            title_color = &CONFIG.os.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.os.title_bold).is_some() {
            title_bold = CONFIG.os.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.os.title_italic).is_some() {
            title_italic = CONFIG.os.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.os.seperator.is_some() {
            seperator = CONFIG.os.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.os.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.os.format.replace("{distro}", &self.distro)
            .replace("{kernel}", &self.kernel)
    }
}
impl Display for OSInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.distro, self.kernel)
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
            log_error("OS", format!("Can't read from /etc/os-release - {}", e));
            return os
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("OS", format!("Can't read from /etc/os-release - {}", e));
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
            log_error("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e));
            return os
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("OS", format!("Can't read from /proc/sys/kernel/osrelease - {}", e));
            return os
        },
    }
    os.kernel = contents.trim().to_string();

    os
}
