use core::str;
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

pub struct SwapInfo {
    used_kib: u32,
    total_kib: u32,
}
#[derive(Deserialize)]
pub struct SwapConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String
}
impl Module for SwapInfo {
    fn new() -> SwapInfo {
        SwapInfo {
            used_kib: 0,
            total_kib: 0
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.swap.title_color).is_some() {
            title_color = &CONFIG.swap.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.swap.title_bold).is_some() {
            title_bold = CONFIG.swap.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.swap.title_italic).is_some() {
            title_italic = CONFIG.swap.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.swap.seperator.is_some() {
            seperator = CONFIG.swap.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.swap.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.swap.format.replace("{used_kib}", &self.used_kib.to_string())
            .replace("{used_mib}", &(self.used_kib as f32 / 1024.0).round().to_string())
            .replace("{used_gib}", &(self.used_kib as f32 / 1024.0 / 1024.0).round().to_string())
            .replace("{total_kib}", &self.total_kib.to_string())
            .replace("{total_mib}", &(self.total_kib as f32 / 1024.0).round().to_string())
            .replace("{total_gib}", &(self.total_kib as f32 / 1024.0 / 1024.0).round().to_string())
            .replace("{percent}", &SwapInfo::round((self.used_kib as f32 / self.total_kib as f32) * 100.0, 2).to_string())
    }
}

pub fn get_swap() -> SwapInfo {
    let mut swap: SwapInfo = SwapInfo::new();

    // Uses /proc/swaps
    let mut file: File = match File::open("/proc/swaps") {
        Ok(r) => r,
        Err(e) => {
            log_error("Swap", format!("Can't read from /proc/swaps - {}", e));
            return swap
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Swap", format!("Can't read from /proc/swaps - {}", e));
            return swap
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

        swap.used_kib += values[3].parse::<u32>().unwrap();
        swap.total_kib += values[2].parse::<u32>().unwrap();
    }

    swap
}
