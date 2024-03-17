// TODO: Parse /proc/mounts and filter.

use core::str;
use std::{fmt::Display, fs::File, io::Read};

use crate::Module;

pub struct MountInfo {
    device: String,     // /dev/sda
    mount: String,      // /hdd
    filesystem: String, // ext4
    flags: Vec<String>
}
impl MountInfo {
    fn from(value: &str) -> Self {
        // Parses from a /proc/mounts entry
        let values: Vec<&str> = value.split(" ").collect();
        MountInfo {
            device: values[0].to_string(),
            mount: values[1].to_string(),
            filesystem: values[2].to_string(),
            flags: values[3].split(",").map(|x| x.to_string()).collect()
        }
    }
}
impl Module for MountInfo {
    fn new() -> MountInfo {
        MountInfo {
            device: "".to_string(),
            mount: "".to_string(),
            filesystem: "".to_string(),
            flags: vec![]
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        // TODO
        let mut flag_str = String::new();
        self.flags.iter().for_each(|f| flag_str.push_str(f));

        format.replace("{device}", &self.device)
        .replace("{mount}", &self.mount)
        .replace("{filesystem}", &self.filesystem)
        .replace("{flags}", &flag_str)
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
    // This parses /proc/mounts for the info
    // This is filtered out by two rules;
    // - It can't have the "nodev" flag
    // - The device has to begin with "/"
    let mut file: File = match File::open("/proc/mounts") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /proc/mounts - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/mounts - {}", e);
        },
    }
    let entries: Vec<&str> = contents.split("\n").collect::<Vec<&str>>();
    for entry in entries {
        if entry.trim() == "" {
            continue;
        }
        let mount: MountInfo = MountInfo::from(entry);
        if mount.flags.contains(&"nodev".to_string()) || !mount.device.starts_with("/") {
            continue
        }

        mounted_drives.insert(mounted_drives.len(), mount);
    }
}
