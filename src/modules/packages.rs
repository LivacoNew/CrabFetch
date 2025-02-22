use core::str;
use std::fs::{read_dir, ReadDir};

use colored::{ColoredString, Colorize};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, common_sources::package_managers::{self, MANAGER_DPKG, MANAGER_HOMEBREW, MANAGER_PACMAN, MANAGER_XBPS}};

pub struct PackagesInfo {
    packages: Vec<ManagerInfo>
}
#[derive(Deserialize, JsonSchema)]
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

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.packages.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.packages.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.packages.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.packages.separator.as_ref().unwrap_or(&config.separator);

        // Full style
        let mut title_final: String = String::new();

        // Title
        if !config.packages.title.trim().is_empty() {
            let mut title: ColoredString = title_color.color_string(&config.packages.title);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            title_final.push_str(&title.to_string());
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

        let mut format_final: String = separator.to_string();
        format_final.push_str(&self.replace_color_placeholders(&value, config));

        (title_final, format_final)
    }

    fn replace_placeholders(&self, _: &str, _: &Configuration) -> String {
        // done in style() instead
        unimplemented!()
    }

    fn unknown_output(_config: &Configuration) -> (String, String) {
        // get_packages can't fail, so this isn't implemented
        // if it does, your fucked, and panic time ensures
        panic!("Packages should never fail, something's wrong. Report this to my GitHub please.");
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on packages module. This should never happen, please make a bug report!")
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

    packages.packages.push(ManagerInfo::fill("pacman", package_managers.find_all_packages_from(MANAGER_PACMAN).values().len() as u64));
    packages.packages.push(ManagerInfo::fill("dpkg", package_managers.find_all_packages_from(MANAGER_DPKG).values().len() as u64));
    packages.packages.push(ManagerInfo::fill("xbps", package_managers.find_all_packages_from(MANAGER_XBPS).values().len() as u64));
    packages.packages.push(ManagerInfo::fill("brew", package_managers.find_all_packages_from(MANAGER_HOMEBREW).values().len() as u64));

    if let Some(r) = process_flatpak_packages() {
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

pub fn process_flatpak_packages() -> Option<u64> {
    // This counts everything in /app and /runtime
    // This does NOT get full information, as I don't care enough about flatpak to figure out
    // how to process it. It's simply used in the packages module and nowhere else for now
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
