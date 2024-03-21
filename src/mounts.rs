use core::str;
use std::{fmt::Display, process::Command};

use crate::{config_manager::Configuration, Module};

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    space_used_mb: u64,
    space_avail_mb: u64,
    space_total_mb: u64,
    percent: u8
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
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{device}", &self.device)
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
impl Display for MountInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) w/ {} / {}", self.device, self.mount, self.space_used_mb, self.space_total_mb)
    }
}
impl MountInfo {
    pub fn is_ignored(&self, config: &Configuration) -> bool {
        for x in config.mount_ignored.to_vec() {
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
    let contents: String = String::from_utf8(Command::new("df")
            .args(["-x", "tmpfs", "-x", "efivarfs", "-x", "devtmpfs"])
            .output().expect("Unable to run 'df' command.").stdout
        ).expect("Unable to parse 'df' output");

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
