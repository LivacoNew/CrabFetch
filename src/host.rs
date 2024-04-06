use core::str;
use std::{fmt::Display, fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.host.title_color).is_some() {
            title_color = &CONFIG.host.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.host.title_bold).is_some() {
            title_bold = CONFIG.host.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.host.title_italic).is_some() {
            title_italic = CONFIG.host.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.host.seperator.is_some() {
            seperator = CONFIG.host.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.host.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        let mut format: String = "{host}".to_string();
        if CONFIG.host.format.is_some() {
            format = CONFIG.host.format.clone().unwrap();
        }

        format.replace("{host}", &self.host)
    }
}
impl Display for HostInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.host)
    }
}

pub fn get_host() -> HostInfo {
    let mut host: HostInfo = HostInfo::new();

    // Uses /sys/devices/virtual/dmi/id/board_name
    let mut file: File = match File::open("/sys/devices/virtual/dmi/id/board_name") {
        Ok(r) => r,
        Err(e) => {
            log_error("Host", format!("Can't read from /sys/devices/virtual/dmi/id/board_name - {}", e));
            return host
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Host", format!("Can't read from /sys/devices/virtual/dmi/id/board_name - {}", e));
            return host
        },
    }

    host.host = contents.trim().to_string();

    host
}
