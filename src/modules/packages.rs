use core::str;
use std::{env, fs::{read_dir, ReadDir}, path::{Path, PathBuf}};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, common_sources::package_managers::{self, MANAGER_DPKG, MANAGER_HOMEBREW, MANAGER_PACMAN, MANAGER_XBPS}};

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

    if let (Some(s), Some(u)) = process_flatpak_packages() {
        packages.packages.push(ManagerInfo::fill("flatpak-system", s));
        packages.packages.push(ManagerInfo::fill("flatpak-user", u));
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

pub fn process_flatpak_packages() -> (Option<u64>, Option<u64>) {
    // This counts everything in /app and /runtime
    // This does NOT get full information, as I don't care enough about flatpak to figure out
    // how to process it. It's simply used in the packages module and nowhere else for now

    // System
    let system = match search_flatpak_base(Path::new("/var/lib/flatpak")) {
        Ok(r) => Some(r as u64),
        Err(_) => None
    };
    // User
    let home_dir_str: String = match env::var("HOME") {
        Ok(r) => r,
        Err(_) => return (system, None)
    };
    let flatpak_path: PathBuf = Path::new(&home_dir_str).join(".local/share/flatpak");
    let user = match search_flatpak_base(&flatpak_path) {
        Ok(r) => Some(r as u64),
        Err(_) => None,
    };

    (system, user)
}
    
fn search_flatpak_base(base_dir: &Path) -> Result<usize, String>{
    // Credit to
    // https://www.reddit.com/r/flatpak/comments/j23lra/comment/g7301ob/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
    // for explaining the difference between these two

    let mut result: usize = 0;

    // System Apps
    let flatpak_apps_dir: ReadDir = match read_dir(base_dir.join("app")) {
        Ok(r) => r,
        Err(_) => return Err("Unable to read from flatpak app folder".to_string()),
    };
    for package in flatpak_apps_dir {
        if package.is_err() {
            continue
        }
        let package = package.unwrap();
        if package.path().to_str().unwrap().to_lowercase().contains("locale") { // ignore "locale" packages
            continue
        }
        result += match flatpak_count_package_versions(&package.path()) {
            Ok(r) => r,
            Err(_) => return Err("Unable to count flatpak app packages".to_string()),
        }
    }

    // System Runtime
    let flatpak_runtime_dir: ReadDir = match read_dir(base_dir.join("runtime")) {
        Ok(r) => r,
        Err(_) => return Err("Unable to read from flatpak runtime folder".to_string()),
    };
    for package in flatpak_runtime_dir {
        if package.is_err() {
            continue
        }
        let package = package.unwrap();
        if package.path().to_str().unwrap().to_lowercase().contains("locale") { // ignore "locale" packages
            continue
        }
        result += match flatpak_count_package_versions(&package.path()) {
            Ok(r) => r,
            Err(_) => return Err("Unable to count flatpak runtime packages".to_string()),
        }
    }

    Ok(result)
}
fn flatpak_count_package_versions(path: &Path) -> Result<usize, String> {
    let package_dir: ReadDir = match path.read_dir() {
        Ok(r) => r,
        Err(_) => return Err("Unable to read flatpak package directory.".to_string()),
    };
    let mut count: usize = 0;
    for arch in package_dir {
        if arch.is_err() {
            continue
        }
        let arch = arch.unwrap().path();
        // ignore mounted stuff, for some reason needs to be .contains 
        if arch.to_str().unwrap().to_lowercase().contains("current") {
            continue
        }
        let arch_dir: ReadDir = match read_dir(arch) {
            Ok(r) => r,
            Err(_) => return Err("Unable to read flatpak arch directory.".to_string()),
        };
        count += arch_dir.count();
    }

    Ok(count)
}
