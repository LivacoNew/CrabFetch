use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::Deserialize;

use crate::{formatter::{self, CrabFetchColor}, config_manager::Configuration, Module, ModuleError};

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
    pub format: String,
    pub progress_left_border: Option<String>,
    pub progress_right_border: Option<String>,
    pub progress_progress: Option<String>,
    pub progress_empty: Option<String>,
    pub progress_target_length: Option<u8>,
    pub decimal_places: Option<u32>
}
impl Module for MemoryInfo {
    fn new() -> MemoryInfo {
        MemoryInfo {
            used_kib: 0,
            max_kib: 0,
            percentage: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.memory.title_color).is_some() {
            title_color = &config.memory.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.memory.title_bold.is_some() {
            title_bold = config.memory.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.memory.title_italic.is_some() {
            title_italic = config.memory.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.memory.seperator.is_some() {
            seperator = config.memory.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.memory.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.memory.title_color).is_some() {
            title_color = config.memory.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.memory.title_bold.is_some() {
            title_bold = config.memory.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.memory.title_italic.is_some() {
            title_italic = config.memory.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.memory.seperator.is_some() {
            seperator = config.memory.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.memory.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut dec_places: u32 = config.decimal_places;
        if config.memory.decimal_places.is_some() {
            dec_places = config.memory.decimal_places.unwrap();
        }

        let mut bar: String = String::new();
        if config.memory.format.contains("{bar}") {
            let mut left_border: &str = config.progress_left_border.as_str();
            if config.memory.progress_left_border.is_some() {
                left_border = config.memory.progress_left_border.as_ref().unwrap();
            }
            let mut right_border: &str = config.progress_right_border.as_str();
            if config.memory.progress_right_border.is_some() {
                right_border = config.memory.progress_right_border.as_ref().unwrap();
            }
            let mut progress: &str = config.progress_progress.as_str();
            if config.memory.progress_progress.is_some() {
                progress = config.memory.progress_progress.as_ref().unwrap();
            }
            let mut empty: &str = config.progress_empty.as_str();
            if config.memory.progress_empty.is_some() {
                empty = config.memory.progress_empty.as_ref().unwrap();
            }
            let mut length: u8 = config.progress_target_length;
            if config.memory.progress_target_length.is_some() {
                length = config.memory.progress_target_length.unwrap();
            }

            bar.push_str(left_border);

            let bar_length: u8 = length - 2;
            for x in 0..(bar_length) {
                if self.percentage as u8 > ((x as f32 / bar_length as f32) * 100.0) as u8 {
                    bar.push_str(progress);
                } else {
                    bar.push_str(empty);
                }
            }
            bar.push_str(right_border);
        }

        formatter::process_percentage_placeholder(&config.memory.format, MemoryInfo::round(self.percentage, dec_places), &config)
            .replace("{phys_used_kib}", &MemoryInfo::round(self.used_kib as f32, dec_places).to_string())
            .replace("{phys_used_mib}", &MemoryInfo::round(self.used_kib as f32 / 1024.0, dec_places).to_string())
            .replace("{phys_used_gib}", &MemoryInfo::round(self.used_kib as f32 / 1.049e+5, dec_places).to_string())
            .replace("{phys_max_kib}", &MemoryInfo::round(self.max_kib as f32, dec_places).to_string())
            .replace("{phys_max_mib}", &MemoryInfo::round(self.max_kib as f32 / 1024.0, dec_places).to_string())
            .replace("{phys_max_gib}", &MemoryInfo::round(self.max_kib as f32 / 1.049e+5, dec_places).to_string())
            .replace("{bar}", &bar.to_string())
    }
}

pub fn get_memory() -> Result<MemoryInfo, ModuleError> {
    let mut memory: MemoryInfo = MemoryInfo::new();

    // Fetches from /proc/meminfo
    let file: File = match File::open("/proc/meminfo") {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Memory", format!("Can't read from /proc/meminfo - {}", e)));
        },
    };

    let mut mem_available: u32 = 0;
    let buffer: BufReader<File> = BufReader::new(file);
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.starts_with("MemTotal") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            memory.max_kib = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    return Err(ModuleError::new("Memory", format!("Could not parse total memory: {}", e)));
                }
            }
        }
        if line.starts_with("MemAvailable") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = &var[..var.len() - 4].trim();
            mem_available = match var.to_string().parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    return Err(ModuleError::new("Memory", format!("Could not parse memfree memory: {}", e)));
                }
            }
        }
        if memory.max_kib != 0 && mem_available != 0 {
            break;
        }
    }

    memory.used_kib = memory.max_kib - mem_available;
    memory.percentage = (memory.used_kib as f32 / memory.max_kib as f32) * 100.0;

    Ok(memory)
}
