use std::fs::File;
use std::io::Read;

use serde::Deserialize;

use crate::config_manager::CrabFetchColor;
use crate::{log_error, Module, CONFIG};

pub struct MemoryInfo {
    used_kib: u32,
    max_kib: u32,
    percentage: f32
}
#[derive(Deserialize)]
pub struct MemoryConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String
}
impl Module for MemoryInfo {
    fn new() -> MemoryInfo {
        MemoryInfo {
            used_kib: 0,
            max_kib: 0,
            percentage: 0.0
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.memory.title_color).is_some() {
            title_color = &CONFIG.memory.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.memory.title_bold).is_some() {
            title_bold = CONFIG.memory.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.memory.title_italic).is_some() {
            title_italic = CONFIG.memory.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.memory.seperator.is_some() {
            seperator = CONFIG.memory.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.memory.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.memory.format.replace("{phys_used_kib}", &MemoryInfo::round(self.used_kib as f32, 2).to_string())
            .replace("{phys_used_mib}", &MemoryInfo::round(self.used_kib as f32 / 1024.0, 2).to_string())
            .replace("{phys_used_gib}", &MemoryInfo::round(self.used_kib as f32 / 104857.0, 2).to_string())
            .replace("{phys_max_kib}", &MemoryInfo::round(self.max_kib as f32, 2).to_string())
            .replace("{phys_max_mib}", &MemoryInfo::round(self.max_kib as f32 / 1024.0, 2).to_string())
            .replace("{phys_max_gib}", &MemoryInfo::round(self.max_kib as f32 / 104857.0, 2).to_string())
            .replace("{percent}", &MemoryInfo::round(self.percentage, 2).to_string())
    }
}

pub fn get_memory() -> MemoryInfo {
    let mut memory: MemoryInfo = MemoryInfo::new();

    // Fetches from /proc/meminfo
    let mut file: File = match File::open("/proc/meminfo") {
        Ok(r) => r,
        Err(e) => {
            log_error("Memory", format!("Can't read from /proc/meminfo - {}", e));
            return memory
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Memory", format!("Can't read from /proc/meminfo - {}", e));
            return memory
        },
    }


    let entry: &str = contents.split("\n\n").collect::<Vec<&str>>()[0];
    let lines: Vec<&str> = entry.split("\n").collect();

    // MemUsed = Memtotal + Shmem - MemFree - Buffers - Cached - SReclaimable
    // From https://github.com/dylanaraps/neofetch/blob/master/neofetch#L2676
    let mut shmem: u32 = 0;
    let mut mem_free: u32 = 0;
    let mut buffers: u32 = 0;
    let mut cached: u32 = 0;
    let mut s_reclaimable: u32 = 0;

    for line in lines {
        if line.starts_with("MemTotal") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            memory.max_kib = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse total memory: {}", e));
                    0
                }
            }
        }
        if line.starts_with("MemFree") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            mem_free = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse free memory: {}", e));
                    0
                }
            }
        }
        if line.starts_with("Shmem:") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            shmem = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse memory shmem: {}", e));
                    0
                }
            }
        }
        if line.starts_with("Buffers") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            buffers = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse memory buffers: {}", e));
                    0
                }
            }
        }
        if line.starts_with("Cached") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            cached = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse cached memory: {}", e));
                    0
                }
            }
        }
        if line.starts_with("SReclaimable") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            s_reclaimable = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Memory", format!("Could not parse sreclaimable memory: {}", e));
                    0
                }
            }
        }
    }

    // MemUsed = Memtotal + Shmem - MemFree - Buffers - Cached - SReclaimable
    memory.used_kib = memory.max_kib + shmem - mem_free - buffers - cached - s_reclaimable;
    memory.percentage = (memory.used_kib as f32 / memory.max_kib as f32) * 100.0;

    memory
}
