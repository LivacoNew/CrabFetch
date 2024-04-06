use core::str;
use std::{fmt::Display, env, fs::File, io::Read};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{config_manager::{self, CrabFetchColor}, log_error, Module, CONFIG};

pub struct HostnameInfo {
    username: String,
    hostname: String,
}
#[derive(Deserialize)]
pub struct HostnameConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub format: String
}
impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "".to_string(),
            hostname: "".to_string(),
        }
    }
    fn style(&self) -> String {
        let mut str: String = String::new();
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.hostname.title_color).is_some() {
            title_color = &CONFIG.hostname.title_color.as_ref().unwrap();
        }

        let mut title: ColoredString = config_manager::color_string(&CONFIG.hostname.title, title_color);
        if title.trim() != "" {
            if CONFIG.title_bold {
                title = title.bold();
            }
            if CONFIG.title_italic {
                title = title.italic();
            }
            str.push_str(&title.to_string());
            str.push_str(&CONFIG.seperator);
        }
        let mut value: String = self.replace_placeholders();
        value = HostnameInfo::replace_color_placeholders(&value);
        str.push_str(&value.to_string());
        str
    }
    fn replace_placeholders(&self) -> String {
        CONFIG.hostname.format.replace("{username}", &self.username)
            .replace("{hostname}", &self.hostname)
            .to_string()
    }
}
impl Display for HostnameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.username, self.hostname)
    }
}
impl HostnameInfo {
    pub fn format_colored(&self, format: &str, _: u32, color: &CrabFetchColor) -> String {
        format.replace("{hostname}", &config_manager::color_string(&self.hostname, &color).to_string())
            .replace("{username}", &config_manager::color_string(&self.username, &color).to_string())
    }
}

pub fn get_hostname() -> HostnameInfo {
    let mut hostname: HostnameInfo = HostnameInfo::new();

    // Gets the username from $USER
    // Gets the hostname from /etc/hostname
    hostname.username = match env::var("USER") {
        Ok(r) => r,
        Err(e) => {
            log_error("Hostname", format!("WARNING: Could not parse $USER env variable: {}", e));
            "user".to_string()
        }
    };

    // Hostname
    let mut file: File = match File::open("/etc/hostname") {
        Ok(r) => r,
        Err(e) => {
            log_error("Hostname", format!("Can't read from /etc/hostname - {}", e));
            return hostname
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Hostname", format!("Can't read from /etc/hostname - {}", e));
            return hostname
        },
    }
    hostname.hostname = contents.trim().to_string();

    hostname
}
