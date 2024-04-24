use core::str;
use std::{fs::{self, read_dir, File, ReadDir}, io::Read, path::Path};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{config_manager::{self, Configuration, CrabFetchColor}, Module};

pub struct PackagesInfo {
    packages: Vec<ManagerInfo>
}
#[derive(Deserialize)]
pub struct PackagesConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String
}
impl Module for PackagesInfo {
    fn new() -> PackagesInfo {
        PackagesInfo {
            packages: Vec::new()
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.packages.title_color).is_some() {
            title_color = &config.packages.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.packages.title_bold.is_some() {
            title_bold = config.packages.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.packages.title_italic.is_some() {
            title_italic = config.packages.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.packages.seperator.is_some() {
            seperator = config.packages.seperator.as_ref().unwrap();
        }


        // Full style
        let mut str: String = String::new();

        // Title
        if !config.packages.title.trim().is_empty() {
            let mut title: ColoredString = config_manager::color_string(&config.packages.title, title_color);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            str.push_str(&title.to_string());
            // Inline value stuff
            if config.inline_values {
                for _ in 0..(max_title_length - (title.len() as u64)) {
                    str.push_str(" ");
                }
            }
            str.push_str(seperator);
        }

        let mut value: String = String::new();
        for manager in &self.packages {
            if value.len() > 0 {
                value.push_str(", ");
            }
            // :(
            value.push_str(&config.packages.format.replace("{manager}", &manager.manager_name)
                .replace("{count}", &manager.package_count.to_string()));
        }
        value = self.replace_color_placeholders(&value);
        str.push_str(&value.to_string());

        str
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        unimplemented!()
    }
}

pub struct ManagerInfo {
    manager_name: String,
    package_count: u64
}
impl ManagerInfo {
    fn fill(manager_name: &str, package_count: u64) -> ManagerInfo {
        ManagerInfo {
            manager_name: manager_name.to_string(),
            package_count
        }
    }
}

pub fn get_packages() -> PackagesInfo {
    let mut packages: PackagesInfo = PackagesInfo::new();

    match process_pacman_packages() {
        Some(r) => packages.packages.push(ManagerInfo::fill("pacman", r)),
        None => {}
    };
    match process_flatpak_packages() {
        Some(r) => packages.packages.push(ManagerInfo::fill("flatpak", r)),
        None => {}
    };
    match process_dpkg_packages() {
        Some(r) => packages.packages.push(ManagerInfo::fill("dpkg", r)),
        None => {}
    };
    match process_rpm_packages() {
        Some(r) => packages.packages.push(ManagerInfo::fill("rpm", r)),
        None => {}
    };

    packages
}

// Credit for Pacman, Flatpak and dpkg detection goes to FastFetch, they were big brain while I was running pacman -Q like a dummy
fn process_pacman_packages() -> Option<u64> {
    let dir: ReadDir = match read_dir("/var/lib/pacman/local") {
        Ok(r) => r,
        Err(_) => return None,
    };

    Some(dir.count() as u64)
}
fn process_flatpak_packages() -> Option<u64> {
    // This counts everything in /app and /runtime
    let mut result: usize = 0;

    let flatpak_apps_dir: ReadDir = match read_dir("/var/lib/flatpak/app") {
        Ok(r) => r,
        Err(_) => return None,
    };
    result += flatpak_apps_dir.count();

    let flatpak_runtime_dir: ReadDir = match read_dir("/var/lib/flatpak/runtime") {
        Ok(r) => r,
        Err(_) => return None,
    };
    result += flatpak_runtime_dir.count();

    Some(result as u64)
}
fn process_dpkg_packages() -> Option<u64> {
    // This counts all the ok entries in /var/lib/dpkg/status
    // This is extremely highly over-optimised, I'm comparing the raw bytes opposed to converting
    // to strings or whatever as the file is so large I need as much raw performance as possible
    // All of this took the legacy function (down below) from 7ms to 3ms in this new function
    let mut result: u64 = 0;
    let file_bytes: Vec<u8> = match fs::read("/var/lib/dpkg/status") {
        Ok(r) => r,
        Err(_) => return None,
    };
    let target_bytes: Vec<u8> = vec![83, 116, 97, 116, 117, 115, 58, 32, 105, 110, 115, 116, 97, 108, 108, 32, 111, 107, 32, 105, 110, 115, 116, 97, 108, 108, 101, 100];

    let mut count = 0;
    for y in file_bytes {
        if y == target_bytes[count] {
            count += 1;
            if count == (target_bytes.len() - 1) {
                result += 1;
                count = 0;
            }
        } else {
            count = 0;
        }
    }

    // Some(_process_dpkg_packages_legacy().unwrap())
    Some(result)
}
// This does the same as above, but in a more sensible way
// This is just for me checking if any changes I've done to the above are valid, as I don't trust
// myself nevermind my code
fn _process_dpkg_packages_legacy() -> Option<u64> {
    // This counts all the ok entries in /var/lib/dpkg/status
    let dpkg_status_path: &Path = Path::new("/var/lib/dpkg/status");
    if !dpkg_status_path.exists() {
        return None
    }

    let mut file: File = match File::open(dpkg_status_path) {
        Ok(r) => r,
        Err(_) => return None,
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => return None,
    }

    let mut result: u64 = 0;
    for entry in contents.split("\n") {
        if !entry.contains("Status: install ok installed") {
            continue
        }
        result += 1;
    }

    Some(result)
}

fn process_rpm_packages() -> Option<u64> {
    let mut result: u64 = 0;

    // Expected in my test env: 1981
    // Grabs from /var/lib/rpm/rpmdb.sqlite
    let db: sqlite::Connection = match sqlite::open("/var/lib/rpm/rpmdb.sqlite") {
        Ok(r) => r,
        Err(_) => return None,
    };
    match db.iterate("SELECT COUNT(1) FROM `Packages`;", |v| {
        if v[0].1.is_some() {
            result = match v[0].1.unwrap().parse() {
                Ok(r) => r,
                Err(_) => return false,
            };
        }
        true
    }) {
        Ok(_) => Some(result),
        Err(_) => None,
    }
}
