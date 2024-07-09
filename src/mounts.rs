use std::{fs::{self, File}, io::{BufRead, BufReader}, path::{Path, PathBuf}};
use std::mem;

use libc::statfs;
use serde::Deserialize;

use crate::{formatter::{self, CrabFetchColor}, config_manager::Configuration, Module, ModuleError};

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    filesystem: String,
    space_avail_kb: u64,
    space_total_kb: u64,
    percent: f32
}
#[derive(Deserialize)]
pub struct MountConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub progress_left_border: Option<String>,
    pub progress_right_border: Option<String>,
    pub progress_progress: Option<String>,
    pub progress_empty: Option<String>,
    pub progress_target_length: Option<u8>,
    pub decimal_places: Option<u32>,
    pub use_ibis: Option<bool>,
    pub ignore: Vec<String>
}
impl Module for MountInfo {
    fn new() -> MountInfo {
        MountInfo {
            device: "".to_string(),
            mount: "".to_string(),
            filesystem: "".to_string(),
            space_avail_kb: 0,
            space_total_kb: 0,
            percent: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.mounts.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.mounts.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.mounts.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.mounts.seperator.as_ref().unwrap_or(&config.seperator);

        let title: String = config.mounts.title.clone()
            .replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{filesystem}", &self.filesystem);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));


        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.mounts.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.mounts.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.mounts.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.mounts.seperator.as_ref().unwrap_or(&config.seperator);

        let title: String = config.mounts.title.clone()
            .replace("{device}", "Unknown")
            .replace("{mount}", "Unknown")
            .replace("{filesystem}", "Unknown");

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let dec_places: u32 = config.mounts.decimal_places.unwrap_or(config.decimal_places);
        let use_ibis: bool = config.memory.use_ibis.unwrap_or(config.use_ibis);

        let mut bar: String = String::new();
        if config.memory.format.contains("{bar}") {
            let left_border: &str = config.mounts.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.mounts.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.mounts.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.mounts.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.mounts.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percent, length);
        }

        formatter::process_percentage_placeholder(&config.mounts.format, MountInfo::round(self.percent, dec_places), config)
            .replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{filesystem}", &self.filesystem)
            .replace("{space_used}", &formatter::auto_format_bytes(self.space_total_kb - self.space_avail_kb, use_ibis, 0))
            .replace("{space_avail}", &formatter::auto_format_bytes(self.space_avail_kb, use_ibis, dec_places))
            .replace("{space_total}", &formatter::auto_format_bytes(self.space_total_kb, use_ibis, dec_places))
            .replace("{bar}", &bar.to_string())
    }
}
impl MountInfo {
    pub fn is_ignored(&self, config: &Configuration) -> bool {
        for x in &config.mounts.ignore {
            if self.mount.starts_with(x) {
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
        if line.starts_with('#') || line.trim().is_empty() {
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
        let fs: &str = entries[2];
        if fs == "swap" {
            continue
        }

        let mut mount: MountInfo = MountInfo::new();
        mount.mount = mount_point.to_string();
        mount.filesystem = fs.to_string();

        // Convert the device entries to device names
        // TODO: support LABEL and PARTLABEL
        let device_name: &str = entries[0];
        if let Some(uuid) = device_name.strip_prefix("UUID=") {
            // UUID
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
            return Err(ModuleError::new("Mounts", format!("'statfs' syscall failed for mount point {} (code {})", path, x)))
        }

        mount.space_total_kb = (buffer.f_blocks * buffer.f_bsize as u64) / 1000;
        mount.space_avail_kb = (buffer.f_bfree * buffer.f_bsize as u64) / 1000;
        mount.percent = ((((mount.space_total_kb - mount.space_avail_kb) as f64) / mount.space_total_kb as f64) * 100.0) as f32;
    }
    Ok(())
}
