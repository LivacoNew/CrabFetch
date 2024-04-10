use core::str;
use std::{fs::{read_dir, File, ReadDir}, io::Read, path::Path};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{config_manager::{self, CrabFetchColor}, Module, CONFIG, MAX_TITLE_LENGTH};

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

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.packages.title_color).is_some() {
            title_color = &CONFIG.packages.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.packages.title_bold).is_some() {
            title_bold = CONFIG.packages.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.packages.title_italic).is_some() {
            title_italic = CONFIG.packages.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.packages.seperator.is_some() {
            seperator = CONFIG.packages.seperator.as_ref().unwrap();
        }


        // Full style
        let mut str: String = String::new();

        // Title
        if !CONFIG.packages.title.trim().is_empty() {
            let mut title: ColoredString = config_manager::color_string(&CONFIG.packages.title, title_color);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            str.push_str(&title.to_string());
            // Inline value stuff
            if CONFIG.inline_values {
                for _ in 0..(*MAX_TITLE_LENGTH - (title.len() as u64)) {
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
            value.push_str(&CONFIG.packages.format.replace("{manager}", &manager.manager_name)
                .replace("{count}", &manager.package_count.to_string()));
        }
        value = self.replace_color_placeholders(&value);
        str.push_str(&value.to_string());

        str
    }

    fn replace_placeholders(&self) -> String {
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
        Some(r) => {packages.packages.push(ManagerInfo::fill("pacman", r));},
        None => {}
    };
    match process_flatpak_packages() {
        Some(r) => {packages.packages.push(ManagerInfo::fill("flatpak", r));},
        None => {}
    };
    match process_dpkg_packages() {
        Some(r) => {packages.packages.push(ManagerInfo::fill("dpkg", r));},
        None => {}
    };

    packages
}

// Credit for Pacman, Flatpak and dpkg detection goes to FastFetch, they were big brain while I was running pacman -Q like a dummy
fn process_pacman_packages() -> Option<u64> {
    let pacman_path: &Path = Path::new("/var/lib/pacman/local");
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
fn process_flatpak_packages() -> Option<u64> {
    // This counts everything in /app and /runtime
    let flatpak_apps_path: &Path = Path::new("/var/lib/flatpak/app");
    if !flatpak_apps_path.exists() {
        return None
    }
    let flatpak_apps_dir: ReadDir = match read_dir(flatpak_apps_path) {
        Ok(r) => r,
        Err(_) => {
            return None
        },
    };
    let flatpak_apps: u64 = flatpak_apps_dir.count() as u64;

    let flatpak_runtime_path: &Path = Path::new("/var/lib/flatpak/runtime");
    if !flatpak_runtime_path.exists() {
        return None
    }
    let flatpak_runtime_dir: ReadDir = match read_dir(flatpak_runtime_path) {
        Ok(r) => r,
        Err(_) => {
            return None
        },
    };
    let flatpak_runtime: u64 = flatpak_runtime_dir.count() as u64;

    Some(flatpak_apps + flatpak_runtime)
}
fn process_dpkg_packages() -> Option<u64> {
    // This counts all the ok entries in /var/lib/dpkg/status
    let dpkg_status_path: &Path = Path::new("/var/lib/dpkg/status");
    if !dpkg_status_path.exists() {
        return None
    }

    let mut file: File = match File::open(dpkg_status_path) {
        Ok(r) => r,
        Err(_) => {
            return None
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => {
            return None
        },
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
