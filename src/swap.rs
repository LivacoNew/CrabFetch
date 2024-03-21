use core::str;
use std::{env, fmt::Display, fs::File, io::Read};

use crate::Module;

pub struct SwapInfo {
    used: u32,
    total: u32,
}
impl Module for SwapInfo {
    fn new() -> SwapInfo {
        SwapInfo {
            used: 0,
            total: 0
        }
    }
    fn format(&self, format: &str, float_places: u32) -> String {
        format.replace("{used_kib}", &self.used.to_string())
        .replace("{used_mib}", &(self.used / 1024).to_string())
        .replace("{used_gib}", &(self.used / 1024 / 1024).to_string())
        .replace("{total_kib}", &self.total.to_string())
        .replace("{total_mib}", &(self.total / 1024).to_string())
        .replace("{total_gib}", &(self.total / 1024 / 1024).to_string())
        .replace("{percent}", &SwapInfo::round((self.used as f32 / self.total as f32) * 100.0, float_places).to_string())
    }
}
impl Display for SwapInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {}", self.used, self.total)
    }
}

pub fn get_swap() -> SwapInfo {
    let mut swap = SwapInfo::new();
    get_basic_info(&mut swap);

    swap
}

fn get_basic_info(swap: &mut SwapInfo) {
    // Uses /proc/swaps
    let mut file: File = match File::open("/proc/swaps") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /proc/swaps - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/swaps - {}", e);
        },
    }
    let mut lines: Vec<&str> = contents.split("\n").collect();
    lines.remove(0);
    for line in lines {
        if line.trim() == "" {
            continue;
        }
        let mut values: Vec<&str> = line.split(['\t', ' ']).collect();
        values.retain(|x| x.trim() != "");

        swap.used += values[3].parse::<u32>().unwrap();
        swap.total += values[2].parse::<u32>().unwrap();
    }
}
