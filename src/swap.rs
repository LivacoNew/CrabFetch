use core::str;
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::{self, Configuration, CrabFetchColor, ModuleConfiguration, TOMLParseError}, Module, ModuleError};

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
impl Default for SwapConfiguration {
    fn default() -> Self {
        SwapConfiguration {
            title: "Swap".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{used_gib} GiB / {total_gib} GiB ({percent}%)".to_string()
        }
    }
}
impl ModuleConfiguration for SwapConfiguration {
    fn apply_toml_line(&mut self, key: &str, value: &str) -> Result<(), crate::config_manager::TOMLParseError> {
        match key {
            "title" => self.title = config_manager::toml_parse_string(value)?,
            "title_color" => self.title_color = Some(config_manager::toml_parse_string_to_color(value)?),
            "title_bold" => self.title_bold = Some(config_manager::toml_parse_bool(value)?),
            "title_italic" => self.title_italic = Some(config_manager::toml_parse_bool(value)?),
            "seperator" => self.seperator = Some(config_manager::toml_parse_string(value)?),
            "format" => self.format = config_manager::toml_parse_string(value)?,
            _ => return Err(TOMLParseError::new("Unknown key.".to_string(), Some("Swap".to_string()), Some(key.to_string()), value.to_string()))
        }
        Ok(())
    }
}

impl Module for SwapInfo {
    fn new() -> SwapInfo {
        SwapInfo {
            used_kib: 0,
            total_kib: 0
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.swap.title_color).is_some() {
            title_color = &config.swap.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.swap.title_bold.is_some() {
            title_bold = config.swap.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.swap.title_italic.is_some() {
            title_italic = config.swap.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.swap.seperator.is_some() {
            seperator = config.swap.seperator.as_ref().unwrap();
        }

        self.default_style(config, max_title_length, &config.swap.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let swap_percent: String = if self.total_kib != 0 {
            SwapInfo::round((self.used_kib as f32 / self.total_kib as f32) * 100.0, 2).to_string()
        } else {
            "0".to_string()
        };
        config.swap.format.replace("{used_kib}", &self.used_kib.to_string())
            .replace("{used_mib}", &(self.used_kib as f32 / 1024.0).round().to_string())
            .replace("{used_gib}", &(self.used_kib as f32 / 1024.0 / 1024.0).round().to_string())
            .replace("{total_kib}", &self.total_kib.to_string())
            .replace("{total_mib}", &(self.total_kib as f32 / 1024.0).round().to_string())
            .replace("{total_gib}", &(self.total_kib as f32 / 1024.0 / 1024.0).round().to_string())
            .replace("{percent}", &swap_percent)
    }
}

pub fn get_swap() -> Result<SwapInfo, ModuleError> {
    let mut swap: SwapInfo = SwapInfo::new();

    // Uses /proc/swaps
    let mut file: File = match File::open("/proc/swaps") {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Swap", format!("Can't read from /proc/swaps - {}", e)));
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("Swap", format!("Can't read from /proc/swaps - {}", e)));
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

    Ok(swap)
}
