use core::str;
use std::{fmt::Display, fs::File, io::Read};

use crate::{log_error, Module};

pub struct HostInfo {
    host: String,
}
impl Module for HostInfo {
    fn new() -> HostInfo {
        HostInfo {
            host: "".to_string()
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
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
