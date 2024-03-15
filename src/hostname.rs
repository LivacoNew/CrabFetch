use core::str;
use std::{fmt::Display, env, fs::File, io::Read};

use crate::Module;

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
    fn format(&self, format: &str) -> String {
        format.replace("{hostname}", &self.hostname)
        .replace("{username}", &self.username)
    }
}
impl Display for HostnameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.username, self.hostname)
    }
}

pub fn get_hostname() -> HostnameInfo {
    let mut hostname = HostnameInfo::new();
    get_basic_info(&mut hostname);

    hostname
}

fn get_basic_info(hostname: &mut HostnameInfo) {
    // Gets the username from $USER
    // Gets the hostname from /etc/hostname
    hostname.username = match env::var("USER") {
        Ok(r) => r,
        Err(e) => {
            println!("WARNING: Could not parse $USER env variable: {}", e);
            "user".to_string()
        }
    };

    // Hostname
    let mut file: File = match File::open("/etc/hostname") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /etc/hostname - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /etc/hostname - {}", e);
        },
    }
    hostname.hostname = contents.trim().to_string();
}
