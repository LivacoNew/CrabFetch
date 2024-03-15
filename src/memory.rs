use std::{fmt::Display, fs::File};
use std::io::Read;

use crate::Module;

pub struct MemoryInfo {
    used: u32,
    max: u32,
    percentage: f32
}
impl Module for MemoryInfo {
    fn new() -> MemoryInfo {
        MemoryInfo {
            used: 0,
            max: 0,
            percentage: 0.0
        }
    }
    fn format(&self, format: &str) -> String {
        format.replace("{phys_used_kib}", &self.used.to_string())
        .replace("{phys_used_mib}", &(self.used as f32 / 1024.0).to_string())
        .replace("{phys_used_gib}", &(self.used as f32 / 102400.0).to_string())
        .replace("{phys_max_kib}", &self.max.to_string())
        .replace("{phys_max_mib}", &(self.max as f32 / 1024.0).to_string())
        .replace("{phys_max_gib}", &(self.max as f32 / 102400.0).to_string())
        .replace("{percent}", &self.percentage.to_string())
    }
}
impl Display for MemoryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {}", (self.used as f32 / 102400.0), (self.max as f32 / 102400.0))
    }
}

pub fn get_memory() -> MemoryInfo {
    let mut memory = MemoryInfo::new();
    get_basic_info(&mut memory);
    memory
}

fn get_basic_info(memory: &mut MemoryInfo) {
    // Fetches from /proc/meminfo
    let mut file: File = match File::open("/proc/meminfo") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /proc/meminfo - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/meminfo - {}", e);
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
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            memory.max = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse total memory: {}", e);
                    0
                }
            }
        }
        if line.starts_with("MemFree") {
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            mem_free = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse free memory: {}", e);
                    0
                }
            }
        }
        if line.starts_with("Shmem:") {
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            shmem = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse memory shmem: {}", e);
                    0
                }
            }
        }
        if line.starts_with("Buffers") {
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            buffers = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse memory buffers: {}", e);
                    0
                }
            }
        }
        if line.starts_with("Cached") {
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            cached = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse cached memory: {}", e);
                    0
                }
            }
        }
        if line.starts_with("SReclaimable") {
            let mut var = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            s_reclaimable = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse sreclaimable memory: {}", e);
                    0
                }
            }
        }
    }

    // MemUsed = Memtotal + Shmem - MemFree - Buffers - Cached - SReclaimable
    memory.used = memory.max + shmem - mem_free - buffers - cached - s_reclaimable;
    memory.percentage = (memory.used as f32 / memory.max as f32) * 100.0;
}
