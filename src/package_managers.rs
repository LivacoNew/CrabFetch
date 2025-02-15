// Queries and caches package manager entries to prevent duplicate work between Packages module and
// Version detection 

use std::{collections::HashMap, ffi::OsStr, fs::{read_dir, DirEntry, File, ReadDir}, io::{BufRead, BufReader}, path::PathBuf};

pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub manager: u8
}
impl PackageInfo {
    fn new(name: &str, version: &str, manager: u8) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            manager
        }
    }
}

pub struct ManagerInfo {
    pub available_managers: u8,
    pub packages: HashMap<String, PackageInfo>
}
impl ManagerInfo {
    pub fn new() -> Self {
        Self {
            available_managers: 0,
            packages: HashMap::new()
        }
    }

    pub fn probe_and_cache(&mut self) {
        self.process_pacman_packages();
        self.process_dpkg_packages();
        self.process_xbps_packages();
        self.process_homebrew_packages();
    }

    pub fn find_all_packages_from(&self, manager: u8) -> HashMap<&String, &PackageInfo> {
        self.packages.iter()
            .filter(|x| x.1.manager & manager > 0)
            .collect()
    }

    // Credit for Pacman, Flatpak and DPKG detection goes to FastFetch, they were big brain while I was running pacman -Q like a dummy
    fn process_pacman_packages(&mut self) {
        let dir: ReadDir = match read_dir("/var/lib/pacman/local") {
            Ok(r) => r,
            Err(_) => return,
        };

        for x in dir {
            // Bit messy
            if x.is_err() {
                continue
            }
            let x: DirEntry = x.unwrap();
            if let Ok(filetype) = x.file_type() {
                if !filetype.is_dir() {
                    continue
                }

                let raw_name: &OsStr = &x.file_name();
                let file_name: &str = match raw_name.to_str() {
                    Some(r) => r,
                    None => continue,
                };
                // {name, may include -}-{version}-{rev}
                let package_split: Vec<&str> = file_name.split('-').collect();
                let package_name: &str = &package_split[0..package_split.len() - 2].join("-");
                // Strip -git suffix for AUR packages
                let package_name: &str = match package_name.strip_suffix("-git") {
                    Some(r) => r,
                    None => package_name,
                };
                let package_version: &str = package_split[package_split.len() - 2];
                self.add_package_to_cache(PackageInfo::new(package_name, package_version, MANAGER_PACMAN));
            }
        }

        self.available_managers += MANAGER_PACMAN;
    }
    fn process_dpkg_packages(&mut self) {
        let file_path: &str = if cfg!(not(feature = "android")) { 
            "/var/lib/dpkg/status"
        } else {
            "/data/data/com.termux/files/usr/var/lib/dpkg/status"
        };
        let file: File = match File::open(file_path) {
            Ok(r) => r,
            Err(_) => return,
        };

        let buffer: BufReader<File> = BufReader::new(file);
        let mut cur_package: String = String::new();
        let mut cur_package_info: PackageInfo = PackageInfo::new("", "", MANAGER_DPKG);
        for line in buffer.lines() {
            if line.is_err() {
                continue;
            }
            let line = line.unwrap();
            if line.is_empty() {
                self.add_package_to_cache(cur_package_info);
                cur_package = String::new();
                cur_package_info = PackageInfo::new("", "", MANAGER_DPKG);
                continue;
            }

            if let Some(package_name) = line.strip_prefix("Package: ") {
                if cur_package.is_empty() {
                    cur_package = package_name.to_string();
                    cur_package_info.name = package_name.to_string();
                }
                continue;
            }
            if !cur_package.is_empty() {
                if let Some(version) = line.strip_prefix("Version: ") {
                    // https://developer.bigfix.com/relevance/reference/debian-package-version.html
                    let split: Vec<&str> = version.split(':').collect();
                    let version = match split.get(1) {
                        Some(r) => String::from(*r),
                        None => split[0].to_string()
                    };
                    let final_version: String = version.split('-').next().unwrap().to_string();

                    cur_package_info.version = final_version;
                }
            }
        }

        self.available_managers += MANAGER_DPKG;
    }
    fn process_xbps_packages(&mut self) {
        let file: File = match File::open("/var/db/xbps/pkgdb-0.38.plist") {
            Ok(r) => r,
            Err(_) => return,
        };

        let buffer: BufReader<File> = BufReader::new(file);
        let mut cur_package: String = String::new();
        let mut cur_package_info: PackageInfo = PackageInfo::new("", "", MANAGER_XBPS);

        let mut next_line_is_key: bool = false;
        let mut next_line_is_version_str: bool = false;
        let mut dict_level: u8 = 0;
        for line in buffer.lines() {
            if line.is_err() {
                continue;
            }
            let line = line.unwrap();
            if line.is_empty() {
                continue;
            }
            let line = line.trim();

            if next_line_is_key {
                next_line_is_key = false;
                // New package
                cur_package = match line.strip_prefix("<key>") {
                    Some(r) => r.to_string(),
                    None => continue, // Nvm, move on
                };
                cur_package = match cur_package.strip_suffix("</key>") {
                    Some(r) => r.to_string(),
                    None => continue, // Nvm, move on
                };
                // Nothing else useful here
                continue
            }

            if line == "<dict>" { 
                dict_level += 1;
                if dict_level == 1 {
                    // Start of file 
                    next_line_is_key = true;
                    continue
                }
            }
            if line == "</dict>" { 
                dict_level -= 1;
                if dict_level == 1 {
                    // Exiting package
                    self.add_package_to_cache(cur_package_info);
                    cur_package = String::new();
                    cur_package_info = PackageInfo::new("", "", MANAGER_XBPS);
                    next_line_is_key = true;
                    continue;
                }

                // We're past the important stuff
                if dict_level == 0 {
                    break;
                }
                continue
            }

            if !cur_package.is_empty() {
                // We're in package rn
                if line == "<key>pkgver</key>" {
                    // Next line is the version
                    next_line_is_version_str = true;
                    continue;
                }
                if next_line_is_version_str {
                    let string: String = line[8..line.len() - 9].to_string();
                    // {package-name}-{ver}_{rev} 
                    let no_rev: &str = string.split('_').next().unwrap();
                    let version: &str = no_rev.split('-').last().unwrap();

                    cur_package_info.version = version.to_string();
                    next_line_is_version_str = false;
                }
            }
        }

        self.available_managers += MANAGER_XBPS;
    }

    pub fn process_homebrew_packages(&mut self) {
        let homebrew_dirs = vec![
            PathBuf::from("/home/linuxbrew/.linuxbrew/Cellar"), 
            PathBuf::from("/home/linuxbrew/.linuxbrew/Caskroom")
        ];

        let homebrew_dirs: Vec<PathBuf> = homebrew_dirs.into_iter().filter(|it| it.exists() && it.is_dir()).collect();
        if homebrew_dirs.is_empty() {
            return;
        }

        for homebrew_package_dir in homebrew_dirs {
            homebrew_package_dir.read_dir()
                .expect("Already checked")
                .filter_map(Result::ok)
                .filter_map(|package_dir| {
                    if !package_dir.path().is_dir() {
                        return None;
                    }

                    let name = package_dir.file_name().into_string().ok()?;
                    let versions = package_dir.path().read_dir().unwrap();
                    let mut versions: Vec<String> = versions
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter_map(|it| it.file_name().into_string().ok())
                        .collect();
                    versions.sort();
                    let version = versions.last().expect("There is a version").to_owned();
                    Some(PackageInfo {
                        name,
                        version,
                        manager: MANAGER_HOMEBREW,
                    })
                }).for_each(|package| {
                    self.add_package_to_cache(package);
                });
        };
        
        self.available_managers += MANAGER_HOMEBREW;
    }

    /// Add a package to cache, or if it already exists add the manager to the list
    /// If it adds the manager to the list, it will **not** account for differing versions and will
    /// simply leave it as is.
    fn add_package_to_cache(&mut self, package_info: PackageInfo) {
        if let Some(x) = self.packages.get_mut(&package_info.name) {
            x.manager |= package_info.manager;
            return;
        }

        self.packages.insert(package_info.name.to_string(), package_info);
    }
}

// Const numbers for each package manager supported by CrabFetch
// Intended to be each bit in a u8 as a supported manager
pub const MANAGER_PACMAN: u8 = 1;
pub const MANAGER_DPKG: u8 = 2;
pub const MANAGER_XBPS: u8 = 4;
// pub const MANAGER_RPM: u8 = 8;
// pub const MANAGER_FLATPAK: u8 = 16;
pub const MANAGER_HOMEBREW: u8 = 32;
