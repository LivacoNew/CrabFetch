use core::str;
use std::{process::Command, io::ErrorKind::NotFound};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    space_used_mb: u64,
    space_avail_mb: u64,
    space_total_mb: u64,
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
impl MountInfo {
    fn from(value: &str) -> Self {
        // Parses from a df entry
        let mut values: Vec<&str> = value.split(" ").collect();
        values.retain(|x| x.trim() != "");
        MountInfo {
            device: values[0].to_string(),
            mount: values[5].to_string(),
            space_used_mb: values[2].parse::<u64>().unwrap() / 1024,
            space_avail_mb: values[3].parse::<u64>().unwrap() / 1024,
            space_total_mb: values[1].parse::<u64>().unwrap() / 1024,
            percent: values[4][..values[4].len() - 1].parse::<u8>().unwrap()
        }
    }
}
impl Module for MountInfo {
    fn new() -> MountInfo {
        MountInfo {
            device: "".to_string(),
            mount: "".to_string(),
            space_used_mb: 0,
            space_avail_mb: 0,
            space_total_mb: 0,
            percent: 0
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.mounts.title_color).is_some() {
            title_color = &CONFIG.mounts.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.mounts.title_bold).is_some() {
            title_bold = CONFIG.mounts.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.mounts.title_italic).is_some() {
            title_italic = CONFIG.mounts.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.mounts.seperator.is_some() {
            seperator = CONFIG.mounts.seperator.as_ref().unwrap();
        }

        let mut title: String = CONFIG.mounts.title.clone();
        title = title.replace("{device}", &self.device)
            .replace("{mount}", &self.mount);

        self.default_style(&title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.mounts.format.replace("{device}", &self.device)
            .replace("{mount}", &self.mount)
            .replace("{space_used_mb}", &self.space_used_mb.to_string())
            .replace("{space_avail_mb}", &self.space_avail_mb.to_string())
            .replace("{space_total_mb}", &self.space_total_mb.to_string())
            .replace("{space_used_gb}", &(self.space_used_mb / 1024).to_string())
            .replace("{space_avail_gb}", &(self.space_avail_mb / 1024).to_string())
            .replace("{space_total_gb}", &(self.space_total_mb / 1024).to_string())
            .replace("{percent}", &self.percent.to_string())
    }
}
impl MountInfo {
    pub fn is_ignored(&self) -> bool {
        for x in CONFIG.mounts.ignore.to_vec() {
            if self.mount.starts_with(&x) {
                return true
            }
        }

        false
    }
}

pub fn get_mounted_drives() -> Vec<MountInfo> {
    let mut mounts: Vec<MountInfo> = Vec::new();

    // This uses the "df" command to grab the outputs
    let output: Vec<u8> = match Command::new("df")
        .args(["-x", "tmpfs", "-x", "efivarfs", "-x", "devtmpfs"])
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    log_error("Mounts", format!("Mounts requires the 'df' command, which is not present!"));
                } else {
                    log_error("Mounts", format!("Unknown error while fetching mounts: {}", e));
                }

                return mounts
            },
        };

    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("Mounts", format!("Unknown error while fetching mounts: {}", e));
            return mounts
        },
    };

    // Parse
    let mut entries: Vec<&str> = contents.split("\n").collect::<Vec<&str>>();
    entries.remove(0);
    for entry in entries {
        if entry.trim() == "" {
            continue;
        }
        let mount: MountInfo = MountInfo::from(entry);
        mounts.insert(mounts.len(), mount);
    }

    mounts
}
