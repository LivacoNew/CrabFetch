use core::str;
use std::{env, fs::File, io::Read, process::Command};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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
impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "".to_string(),
            hostname: "".to_string(),
        }
    }
    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.hostname.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.hostname.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, seperator, "Unknown")
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
                return Err(ModuleError::new("Hostname", "Can't find hostname source.".to_string()));
            },
        };

    hostname.hostname = match String::from_utf8(output) {
        Ok(r) => r.trim().to_string(),
        Err(_) => {
            return Err(ModuleError::new("Hostname", "Can't find hostname source.".to_string()));
        },
    };

    Ok(())
}
