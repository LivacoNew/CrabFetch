use core::str;
use std::{fmt::{Debug, Display}, fs::File, io::Read, process::Command};

use crate::Module;

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    space_used: u64,
    space_avail: u64
}
impl MountInfo {
    fn from(value: &str) -> Self {
        // Parses from a /proc/mounts entry
        let mut values: Vec<&str> = value.split(" ").collect();
        values.retain(|x| x.trim() != "");
        println!("{:?}", values);
        MountInfo {
            device: values[0].to_string(),
            mount: values[5].to_string(),
            space_used: values[2].parse::<u64>().unwrap(),
            space_avail: values[3].parse::<u64>().unwrap(),
        }
    }
}
impl Module for MountInfo {
    fn new() -> MountInfo {
        MountInfo {
            device: "".to_string(),
            mount: "".to_string(),
            space_used: 0,
            space_avail: 0
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        // TODO
        let mut flag_str = String::new();

        format.replace("{device}", &self.device)
        .replace("{mount}", &self.mount)
        .replace("{space_used_gb}", &(self.space_used / 1024 / 1024).to_string())
        .replace("{space_avail_gb}", &(self.space_avail / 1024 / 1024).to_string())
        .replace("{space_total_gb}", &((self.space_used + self.space_avail) / 1024 / 1024).to_string())
    }
}
impl Display for MountInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "{}", "")
    }
}

pub fn get_mounted_drives() -> Vec<MountInfo> {
    let mut mounts: Vec<MountInfo> = Vec::new();
    get_basic_info(&mut mounts);

    mounts
}

fn get_basic_info(mounted_drives: &mut Vec<MountInfo>) {
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
        mounted_drives.insert(mounted_drives.len(), mount);
    }
}
