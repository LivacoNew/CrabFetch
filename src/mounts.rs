use std::{fs::{self, File}, io::{BufRead, BufReader}, path::{Path, PathBuf}};
use std::mem;

use libc::statfs;
use serde::Deserialize;

use crate::{config_manager::{self, Configuration, CrabFetchColor, ModuleConfiguration, TOMLParseError}, Module, ModuleError};

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    space_avail_mb: i64,
    space_total_mb: i64,
    percent: u8
}
#[derive(Deserialize)]
pub struct MountConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub ignore: Vec<String>
}
impl Default for MountConfiguration {
    fn default() -> Self {
        MountConfiguration {
            title: "Disk {mount}".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{space_used_gb} GB used of {space_total_gb} GB ({percent}%)".to_string(),
            ignore: vec!["/boot".to_string(), "/snap".to_string()]
        }
    }
}
impl ModuleConfiguration for MountConfiguration {
    fn apply_toml_line(&mut self, key: &str, value: &str) -> Result<(), crate::config_manager::TOMLParseError> {
        match key {
            "title" => self.title = config_manager::toml_parse_string(value)?,
            "title_color" => self.title_color = Some(config_manager::toml_parse_string_to_color(value)?),
            "title_bold" => self.title_bold = Some(config_manager::toml_parse_bool(value)?),
            "title_italic" => self.title_italic = Some(config_manager::toml_parse_bool(value)?),
            "seperator" => self.seperator = Some(config_manager::toml_parse_string(value)?),
            "format" => self.format = config_manager::toml_parse_string(value)?,
            "ignore" => self.ignore = config_manager::toml_parse_str_array(value)?,
            _ => return Err(TOMLParseError::new("Unknown key.".to_string(), Some("Mounts".to_string()), Some(key.to_string()), value.to_string()))
        }
        Ok(())
    }
}


impl Module for MountInfo {
    fn new() -> MountInfo {
        MountInfo {
            device: "".to_string(),
            mount: "".to_string(),
            space_avail_mb: 0,
            space_total_mb: 0,
            percent: 0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.mounts.title_color).is_some() {
            title_color = config.mounts.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.mounts.title_bold.is_some() {
            title_bold = config.mounts.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.mounts.title_italic.is_some() {
            title_italic = config.mounts.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.mounts.seperator.is_some() {
            seperator = config.mounts.seperator.as_ref().unwrap();
        }

        let mut title: String = config.mounts.title.clone();
        title = title.replace("{device}", &self.device)
            .replace("{mount}", &self.mount);

        self.default_style(config, max_title_size, &title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.mounts.format.replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{space_used_mb}", &(self.space_total_mb - self.space_avail_mb).to_string())
            .replace("{space_avail_mb}", &self.space_avail_mb.to_string())
            .replace("{space_total_mb}", &self.space_total_mb.to_string())
            .replace("{space_used_gb}", &((self.space_total_mb - self.space_avail_mb) / 1024).to_string())
            .replace("{space_avail_gb}", &(self.space_avail_mb / 1024).to_string())
            .replace("{space_total_gb}", &(self.space_total_mb / 1024).to_string())
            .replace("{percent}", &self.percent.to_string())
    }
}
impl MountInfo {
    pub fn is_ignored(&self, config: &Configuration) -> bool {
        for x in config.mounts.ignore.to_vec() {
            if self.mount.starts_with(&x) {
                return true
            }
        }

        false
    }
}

pub fn get_mounted_drives() -> Result<Vec<MountInfo>, ModuleError> {
    let mut mounts: Vec<MountInfo> = Vec::new();

    // Read from /etc/fstab to get all currently mounted disks
    let file: File = match File::open("/etc/fstab") {
        Ok(r) => r,
        Err(e) => {
            // Best guess I've got is that we're not on Linux
            // In which case, L
            return Err(ModuleError::new("Mounts", format!("Unable to read from /etc/fstab: {}", e)));
        },
    };
    let buffer: BufReader<File> = BufReader::new(file);
    for line in buffer.lines() {
        if line.is_err() {
            continue
        }
        let line: String = line.unwrap();
        if line.starts_with("#") || line.trim().is_empty() {
            continue
        }

        let entries: Vec<&str> = line.split([' ', '\t'])
            .filter(|x| x.trim() != "")
            .map(|x| x.trim())
            .collect();
        let mount_point: &str = entries[1];
        if mount_point == "none" || mount_point == "swap" {
            continue
        }

        let mut mount: MountInfo = MountInfo::new();
        mount.mount = mount_point.to_string();

        // Convert the device entries to device names
        // TODO: support LABEL and PARTLABEL
        let device_name: &str = entries[0];
        if device_name.starts_with("UUID=") {
            // UUID
            let uuid: String = device_name[5..].to_string();
            let uuid_path: PathBuf = Path::new("/dev/disk/by-uuid/").join(uuid);
            if !uuid_path.is_symlink() {
                continue; // ??
            }
            let device = match fs::canonicalize(uuid_path) {
                Ok(r) => r,
                Err(_) => continue, // ??
            };
            mount.device = device.to_str().unwrap().to_string();
        } else {
            // regular old devices
            mount.device = device_name.to_string();
        }

        // statfs to get space data
        let statfs: Result<(), ModuleError> = call_statfs(mount_point, &mut mount);
        if statfs.is_err() {
            return Err(ModuleError::new("Mounts", format!("'statfs' syscall failed for mount point {}", mount_point)));
        }

        mounts.push(mount);
    }

    Ok(mounts)
}

// Credit to sysinfo crate for letting me see how to impl this in Rust (and no it's not just copy
// pasted i swear)
// https://github.com/GuillaumeGomez/sysinfo/blob/master/src/unix/linux/disk.rs#L96
fn call_statfs(path: &str, mount: &mut MountInfo) -> Result<(), ModuleError> {
    let mut bytes: Vec<u8> = path.as_bytes().to_vec();
    bytes.push(0);
    unsafe { // spooky
        let mut buffer: statfs = mem::zeroed();
        // wtf does this "*const _" do
        let x: i32 = statfs(bytes.as_ptr() as *const _, &mut buffer);
        if x != 0 {
            // log_error("Mount", format!("'statfs' syscall failed for mount point {path} - Returned code {x}"));
            return Err(ModuleError::new("Mounts", format!("'statfs' syscall failed for mount point {} (code {})", path, x)))
        }

        mount.space_total_mb = ((buffer.f_blocks as i64) * buffer.f_bsize) / 1024 / 1024;
        mount.space_avail_mb = ((buffer.f_bavail as i64) * buffer.f_bsize) / 1024 / 1024;
        mount.percent = ((((mount.space_total_mb - mount.space_avail_mb) as f64) / mount.space_total_mb as f64) * 100.0) as u8;
    }
    Ok(())
}
