use core::str;
use std::{fmt::Display, env, fs::File, io::Read};

use crate::{config_manager::{self, CrabFetchColor}, Module};

pub struct HostnameInfo {
    username: String,
    hostname: String,
}
impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "".to_string(),
            hostname: "".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{hostname}", &self.hostname)
            .replace("{username}", &self.username)
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
            print!("WARNING: Could not parse $USER env variable: {}", e);
            "user".to_string()
        }
    };

    // Hostname
    let mut file: File = match File::open("/etc/hostname") {
        Ok(r) => r,
        Err(e) => {
            print!("Can't read from /etc/hostname - {}", e);
            return hostname
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            print!("Can't read from /etc/hostname - {}", e);
            return hostname
        },
    }
    hostname.hostname = contents.trim().to_string();

    hostname
}
