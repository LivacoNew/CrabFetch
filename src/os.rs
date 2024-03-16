use core::str;
use std::{fmt::Display, fs::File, io::Read};

use crate::Module;

pub struct OSInfo {
    distro: String,
    kernel: String
}
impl Module for OSInfo {
    fn new() -> OSInfo {
        OSInfo {
            distro: "".to_string(),
            kernel: "".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{distro}", &self.distro)
        .replace("{kernel}", &self.kernel)
    }
}
impl Display for OSInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.distro, self.kernel)
    }
}

pub fn get_os() -> OSInfo {
    let mut os = OSInfo::new();
    get_basic_info(&mut os);

    os
}

fn get_basic_info(os: &mut OSInfo) {
    // Grabs the distro name from /etc/os-release
    // Grabs the kernel release from /proc/sys/kernel/osrelease

    // Distro
    let mut file: File = match File::open("/etc/os-release") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /etc/os-release - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /etc/os-release - {}", e);
        },
    }
    for line in contents.trim().to_string().split("\n").collect::<Vec<&str>>() {
        if !line.starts_with("PRETTY_NAME=") {
            continue;
        }
        os.distro = line[13..line.len() - 1].to_string();
    }

    // Kernel
    let mut file: File = match File::open("/proc/sys/kernel/osrelease") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /proc/sys/kernel/osrelease - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/sys/kernel/osrelease - {}", e);
        },
    }
    os.kernel = contents.trim().to_string();
}
