use core::str;

#[cfg(feature = "android")]
use std::env;

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, package_managers::{self, MANAGER_DPKG, MANAGER_PACMAN, MANAGER_XBPS}, Module};

pub struct PackagesInfo {
    packages: Vec<ManagerInfo>
}
#[derive(Deserialize)]
pub struct PackagesConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub ignore: Vec<String>,
    pub format: String
}
impl Module for PackagesInfo {
    fn new() -> PackagesInfo {
        PackagesInfo {
            packages: Vec::new()
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let title_color: &CrabFetchColor = config.packages.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.packages.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.packages.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.packages.separator.as_ref().unwrap_or(&config.separator);

        // Full style
        let mut str: String = String::new();

        // Title
        if !config.packages.title.trim().is_empty() {
            let mut title: ColoredString = title_color.color_string(&config.packages.title);
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
                    str.push(' ');
                }
            }
            str.push_str(separator);
        }

        let mut value: String = String::new();
        for manager in &self.packages {
            if config.packages.ignore.contains(&manager.manager_name) {
                continue
            }
            if manager.package_count == 0 {
                continue
            }

            if !value.is_empty() {
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

    fn replace_placeholders(&self, _: &Configuration) -> String {
        unimplemented!()
    }

    fn unknown_output(_config: &Configuration, _max_title_length: u64) -> String {
        // get_packages can't fail, so this isn't implemented
        // if it does, your fucked, and panic time ensures
        panic!("Packages should never fail, something's wrong. Report this to my GitHub please.");
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

pub fn get_packages(package_managers: &package_managers::ManagerInfo) -> PackagesInfo {
    let mut packages: PackagesInfo = PackagesInfo::new();

    // if let Some(r) = process_pacman_packages() {
    //     packages.packages.push(ManagerInfo::fill("pacman", r));
    // }
    packages.packages.push(ManagerInfo::fill("pacman", package_managers.find_all_packages_from(MANAGER_PACMAN).values().len() as u64));
    packages.packages.push(ManagerInfo::fill("dpkg", package_managers.find_all_packages_from(MANAGER_DPKG).values().len() as u64));
    packages.packages.push(ManagerInfo::fill("xbps", package_managers.find_all_packages_from(MANAGER_XBPS).values().len() as u64));

    if let Some(r) = package_managers.process_flatpak_packages_count() {
        packages.packages.push(ManagerInfo::fill("flatpak", r));
    }
    #[cfg(feature = "rpm_packages")]
    if let Some(r) = process_rpm_packages() {
        packages.packages.push(ManagerInfo::fill("rpm", r));
    }

    packages
}

#[cfg(feature = "rpm_packages")]
fn process_rpm_packages() -> Option<u64> {
    let mut result: u64 = 0;

    // Expected in my test env: 1981
    // Grabs from /var/lib/rpm/rpmdb.sqlite
    let db: sqlite::Connection = match sqlite::open("/var/lib/rpm/rpmdb.sqlite") {
        Ok(r) => r,
        Err(_) => return None,
    };

    let success: Result<_, _> = db.iterate("SELECT COUNT(1) FROM `Packages`;", |v| {
        if v[0].1.is_some() {
            result = match v[0].1.unwrap().parse() {
                Ok(r) => r,
                Err(_) => return false,
            };
        }
        true
    });

    match success {
        Ok(_) => Some(result),
        Err(_) => None,
    }
}
