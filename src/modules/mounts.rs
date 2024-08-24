use std::{fs::{self, File}, io::{BufRead, BufReader, Error}, path::{Path, PathBuf}};
use std::mem;

#[cfg(feature = "android")]
use std::env;

use libc::statfs;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, util::{self, is_flag_set_u32}, ModuleError};

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
    pub separator: Option<String>,
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
            device: "Unknown".to_string(),
            mount: "Unknown".to_string(),
            filesystem: "Unknown".to_string(),
            space_avail_kb: 0,
            space_total_kb: 0,
            percent: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.mounts.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.mounts.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.mounts.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.mounts.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.mounts.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.mounts.format, config));

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.mounts.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.mounts.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.mounts.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.mounts.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.mounts.title
            .replace("{device}", "Unknown")
            .replace("{mount}", "Unknown")
            .replace("{filesystem}", "Unknown")
            .replace("{space_used}", "Unknown")
            .replace("{space_avail}", "Unknown")
            .replace("{space_total}", "Unknown")
            .replace("{bar}", " ");

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let dec_places: u32 = config.mounts.decimal_places.unwrap_or(config.decimal_places);
        let use_ibis: bool = config.mounts.use_ibis.unwrap_or(config.use_ibis);

        let mut bar: String = String::new();
        if text.contains("{bar}") {
            let left_border: &str = config.mounts.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.mounts.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.mounts.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.mounts.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.mounts.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percent, length);
        }

        formatter::process_percentage_placeholder(text, formatter::round(self.percent as f64, dec_places) as f32, config)
            .replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{filesystem}", &self.filesystem)
            .replace("{space_used}", &formatter::auto_format_bytes(self.space_total_kb - self.space_avail_kb, use_ibis, dec_places))
            .replace("{space_avail}", &formatter::auto_format_bytes(self.space_avail_kb, use_ibis, dec_places))
            .replace("{space_total}", &formatter::auto_format_bytes(self.space_total_kb, use_ibis, dec_places))
            .replace("{bar}", &bar.to_string())
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{device}") {
            info_flags |= MOUNTS_INFOFLAG_DEVICE;
        }
        if format.contains("{space_used}") || format.contains("bar") {
            info_flags |= MOUNTS_INFOFLAG_SPACE_USED;
        }
        if format.contains("{space_avail}") {
            info_flags |= MOUNTS_INFOFLAG_SPACE_AVAIL;
        }
        if format.contains("{space_total}") || format.contains("bar") {
            info_flags |= MOUNTS_INFOFLAG_SPACE_TOTAL;
        }

        info_flags
    }
}
impl MountInfo {
    pub fn is_ignored(&self, config: &Configuration) -> bool {
        for x in &config.mounts.ignore {
            if x.is_empty() {
                continue;
            }
            if self.mount.starts_with(x) || self.filesystem.starts_with(x) {
                return true
            }
        }

        false
    }
    // Used by calc_max_title_length
    pub fn get_title_size(&self, config: &Configuration) -> u64 {
        config.mounts.title.clone()
            .replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{filesystem}", &self.filesystem)
            .chars().count() as u64
    }
}

const MOUNTS_INFOFLAG_DEVICE: u32 = 1;
const MOUNTS_INFOFLAG_SPACE_USED: u32 = 4;
const MOUNTS_INFOFLAG_SPACE_TOTAL: u32 = 8;
const MOUNTS_INFOFLAG_SPACE_AVAIL: u32 = 16;

pub fn get_mounted_drives(config: &Configuration) -> Result<Vec<MountInfo>, ModuleError> {
    let mut mounts: Vec<MountInfo> = Vec::new();
    // title is tagged onto the end here to account for the title placeholders
    let info_flags: u32 = MountInfo::gen_info_flags(&format!("{}{}", config.mounts.format, config.mounts.title));

    #[cfg(not(feature = "android"))]
    let path: &str = "/etc/mtab";
    #[cfg(feature = "android")]
    let mut path: &str = "/etc/mtab";
    // Android 
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        path = "/proc/mounts";
    }

    let file: File = match File::open(path) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Mounts", format!("Unable to read from /etc/mtab: {}", e))),
    };
    let buffer: BufReader<File> = BufReader::new(file);
    let mut device_cache: Vec<String> = Vec::new();
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
    
        let mut mount: MountInfo = MountInfo::new();
        let device_name: &str = entries[0];
        mount.device = match get_device_name(device_name) {
            Some(r) => r,
            None => continue, // Invalid device or not a device we want
        };
        // skipped on android as we want fuse
        #[cfg(not(feature = "android"))]
        if device_cache.contains(&mount.device) {
            continue; // Already processed
        }
        device_cache.push(device_name.to_string());

        if !is_device_wanted(&mount.device) {
            continue; // bullshit
        }

        let mount_point: String = entries[1].replace("\\040", " ");
        if mount_point == "none" || mount_point == "swap" {
            continue
        }
        // Android is a odd-ball with a *whitelist* instead as it has way too many weird and
        // wonderful mounts, and as far as I can tell is overall an exception
        #[cfg(feature = "android")]
        if mount_point != "/" && !mount_point.starts_with("/storage") {
            continue
        }

        mount.mount = mount_point.to_string();
        mount.filesystem = entries[2].to_string();
        if mount.is_ignored(config) {
            continue;
        }

        // statfs to get space data
        if is_flag_set_u32(info_flags, MOUNTS_INFOFLAG_SPACE_AVAIL | MOUNTS_INFOFLAG_SPACE_USED | MOUNTS_INFOFLAG_SPACE_TOTAL) {
            call_statfs(&mount_point, &mut mount)?;
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
        let x: i32 = statfs(bytes.as_ptr() as *const _, &mut buffer);
        if x != 0 {
            let c: String = match Error::last_os_error().raw_os_error() {
                Some(r) => r.to_string(),
                None => "N/A".to_string(),
            };
            return Err(ModuleError::new("Mounts", format!("'statfs' syscall failed for mount point {} (code {})", path, c)))
        }

        mount.space_total_kb = (buffer.f_blocks * buffer.f_bsize as u64) / 1000;
        mount.space_avail_kb = (buffer.f_bfree * buffer.f_bsize as u64) / 1000;
        mount.percent = ((((mount.space_total_kb - mount.space_avail_kb) as f64) / mount.space_total_kb as f64) * 100.0) as f32;
    }
    Ok(())
}

fn get_device_name(device_name: &str) -> Option<String> {
    // This method is also responsible for filtering out any devices we don't want
    let dev: String;
    if let Some(uuid) = device_name.strip_prefix("UUID=") {
        // UUID
        let uuid_path: PathBuf = Path::new("/dev/disk/by-uuid/").join(uuid);
        if !uuid_path.is_symlink() {
            return None; // ???
        }
        let device = match fs::canonicalize(uuid_path) {
            Ok(r) => r,
            Err(_) => return None, // ??
        };
        dev = device.to_str().unwrap().to_string();
    } else if let Some(label) = device_name.strip_prefix("LABEL=") {
        let label_path: PathBuf = Path::new("/dev/disk/by-label/").join(label);
        if !label_path.is_symlink() {
            return None; // ???
        }
        let device = match fs::canonicalize(label_path) {
            Ok(r) => r,
            Err(_) => return None, // ??
        };
        dev = device.to_str().unwrap().to_string();
    } else if let Some(partlabel) = device_name.strip_prefix("PARTLABEL=") {
        let label_path: PathBuf = Path::new("/dev/disk/by-partlabel/").join(partlabel);
        if !label_path.is_symlink() {
            return None; // ???
        }
        let device = match fs::canonicalize(label_path) {
            Ok(r) => r,
            Err(_) => return None, // ??
        };
        dev = device.to_str().unwrap().to_string();
    } else {
        // regular old devices
        dev = device_name.to_string();
    }

    Some(dev)
}

fn is_device_wanted(device_name: &str) -> bool {
    // WSL
    // FIXME: This is INCREDIBLY hacky
    if util::in_wsl() && &device_name[1..3] == ":\\" {
        return true;
    }

    // This starts_with is a very primative check for if a device is virtual or physical, aka
    // whether we care about it or not
    #[cfg(not(feature = "android"))]
    if !device_name.starts_with('/') {
        return false;
    }

    // let in fuse for android
    #[cfg(feature = "android")]
    if !device_name.starts_with('/') && device_name != "fuse" {
        return false;
    }
    
    true
}
