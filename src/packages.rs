use core::str;
use std::{fmt::Display, fs::{read_dir, ReadDir}, path::Path};

use crate::Module;

pub struct PackagesInfo {
    packages: Vec<ManagerInfo>
}
impl Module for PackagesInfo {
    fn new() -> PackagesInfo {
        PackagesInfo {
            packages: Vec::new()
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        let mut str: String = String::new();
        for manager in &self.packages {
            if str.len() > 0 {
                str.push_str(", ");
            }
            str.push_str(&format.replace("{manager}", &manager.manager_name)
                         .replace("{count}", &manager.package_count.to_string()));
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

    packages
}

// Credit for Pacman/Flatpak detection goes to FastFetch, they were big brain while I was running pacman -Q like a dummy
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
