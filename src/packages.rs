use core::str;
use std::{collections::HashMap, fmt::Display, fs::{read_dir, ReadDir}, path::Path};

use crate::Module;

pub struct PackagesInfo {
    packages: HashMap<String, u64>
}
impl Module for PackagesInfo {
    fn new() -> PackagesInfo {
        PackagesInfo {
            packages: HashMap::new()
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        let mut str: String = String::new();
        for (manager, count) in &self.packages {
            str.push_str(&format.replace("{manager}", &manager)
                         .replace("{count}", &count.to_string()));
        }
        str
    }
}
impl Display for PackagesInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "")
    }
}

pub fn get_packages() -> PackagesInfo {
    let mut packages: PackagesInfo = PackagesInfo::new();

    match process_pacman_packages() {
        Some(r) => {packages.packages.insert("pacman".to_string(), r);},
        None => {}
    };

    packages
}

fn process_pacman_packages() -> Option<u64> {
    // Credit to FastFetch here, I was running pacman -Q like a dummy
    // https://github.com/fastfetch-cli/fastfetch/blob/dev/src/detection/packages/packages_linux.c#L256
    let pacman_path = Path::new("/var/lib/pacman/local");
    if !pacman_path.exists() {
        return None
    }

    let dir: ReadDir = match read_dir(pacman_path) {
        Ok(r) => r,
        Err(_) => {
            // Silent error!
            return None
        },
    };

    Some(dir.count() as u64)
}
