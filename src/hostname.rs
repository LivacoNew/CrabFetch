use core::str;
use std::{env, fs::File, io::Read, process::Command};

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

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
    pub seperator: Option<String>,
    pub format: String
}
impl Default for HostnameConfiguration {
    fn default() -> Self {
        HostnameConfiguration {
            title: "".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{color-brightmagenta}{username}{color-white}@{color-brightmagenta}{hostname}".to_string()
        }
    }
}

impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "".to_string(),
            hostname: "".to_string(),
        }
    }
    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.hostname.title_color).is_some() {
            title_color = config.hostname.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.hostname.title_bold.is_some() {
            title_bold = config.hostname.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.hostname.title_italic.is_some() {
            title_italic = config.hostname.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.hostname.seperator.is_some() {
            seperator = config.hostname.seperator.as_ref().unwrap();
        }

        self.default_style(config, max_title_length, &config.hostname.title, title_color, title_bold, title_italic, &seperator)
    }
    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.hostname.format.replace("{username}", &self.username)
            .replace("{hostname}", &self.hostname)
            .to_string()
    }
}

pub fn get_hostname() -> Result<HostnameInfo, ModuleError> {
    let mut hostname: HostnameInfo = HostnameInfo::new();

    // Gets the username from $USER
    // Gets the hostname from /etc/hostname
    hostname.username = match env::var("USER") {
        Ok(r) => r,
        Err(e) => {
           return Err(ModuleError::new("Hostname", format!("WARNING: Could not parse $USER env variable: {}", e)));
        }
    };

    // Hostname
    let mut file: File = match File::open("/etc/hostname") {
        Ok(r) => r,
        Err(_) => {
            match backup_to_hostname_command(&mut hostname) {
                Ok(_) => return Ok(hostname),
                Err(e) => return Err(e),
            }
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => {
            match backup_to_hostname_command(&mut hostname) {
                Ok(_) => return Ok(hostname),
                Err(e) => return Err(e),
            }
        },
    }
    if contents.trim().is_empty() {
        match backup_to_hostname_command(&mut hostname) {
            Ok(_) => return Ok(hostname),
            Err(e) => return Err(e),
        }
    }

    hostname.hostname = contents.trim().to_string();
    Ok(hostname)
}

fn backup_to_hostname_command(hostname: &mut HostnameInfo) -> Result<(), ModuleError> {
    // If /etc/hostname is fucked, it'll come here
    // This method is requried e.g in a default fedora install, where they don't bother filling out
    // the /etc/hostname file
    let output: Vec<u8> = match Command::new("hostname")
        .output() {
            Ok(r) => r.stdout,
            Err(_) => {
                // fuck it
                return Err(ModuleError::new("Hostname", format!("Can't find hostname source.")));
            },
        };

    hostname.hostname = match String::from_utf8(output) {
        Ok(r) => r.trim().to_string(),
        Err(_) => {
            return Err(ModuleError::new("Hostname", format!("Can't find hostname source.")));
        },
    };

    Ok(())
}
