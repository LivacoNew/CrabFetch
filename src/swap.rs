use core::str;
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{formatter::{self, CrabFetchColor}, config_manager::Configuration, Module, ModuleError};

pub struct SwapInfo {
    used_kb: u64,
    total_kb: u64,
    percent: f32
}
#[derive(Deserialize)]
pub struct SwapConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub progress_left_border: Option<String>,
    pub progress_right_border: Option<String>,
    pub progress_progress: Option<String>,
    pub progress_empty: Option<String>,
    pub progress_target_length: Option<u8>,
    pub decimal_places: Option<u32>,
    pub use_ibis: Option<bool>,
    pub format: String
}
impl Module for SwapInfo {
    fn new() -> SwapInfo {
        SwapInfo {
            used_kb: 0,
            total_kb: 0,
            percent: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
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

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.swap.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.swap.title_color).is_some() {
            title_color = config.swap.title_color.as_ref().unwrap();
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

        Self::default_style(config, max_title_size, &config.swap.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut dec_places: u32 = config.decimal_places;
        if config.mounts.decimal_places.is_some() {
            dec_places = config.mounts.decimal_places.unwrap();
        }

        let mut bar: String = String::new();
        if config.swap.format.contains("{bar}") {
            let mut left_border: &str = config.progress_left_border.as_str();
            if config.swap.progress_left_border.is_some() {
                left_border = config.swap.progress_left_border.as_ref().unwrap();
            }
            let mut right_border: &str = config.progress_right_border.as_str();
            if config.swap.progress_right_border.is_some() {
                right_border = config.swap.progress_right_border.as_ref().unwrap();
            }
            let mut progress: &str = config.progress_progress.as_str();
            if config.swap.progress_progress.is_some() {
                progress = config.swap.progress_progress.as_ref().unwrap();
            }
            let mut empty: &str = config.progress_empty.as_str();
            if config.swap.progress_empty.is_some() {
                empty = config.swap.progress_empty.as_ref().unwrap();
            }
            let mut length: u8 = config.progress_target_length;
            if config.swap.progress_target_length.is_some() {
                length = config.swap.progress_target_length.unwrap();
            }

            bar.push_str(left_border);

            let bar_length: u8 = length - 2;
            for x in 0..(bar_length) {
                if self.percent as u8 > ((x as f32 / bar_length as f32) * 100.0) as u8 {
                    bar.push_str(progress);
                } else {
                    bar.push_str(empty);
                }
            }
            bar.push_str(right_border);
        }

        let mut use_ibis: bool = config.use_ibis;
        if config.swap.use_ibis.is_some() {
            use_ibis = config.swap.use_ibis.unwrap();
        }

        formatter::process_percentage_placeholder(&config.swap.format, SwapInfo::round(self.percent, dec_places), &config)
            .replace("{used}", &formatter::auto_format_bytes(self.used_kb, use_ibis, dec_places))
            .replace("{total}", &formatter::auto_format_bytes(self.total_kb, use_ibis, dec_places))
            .replace("{bar}", &bar)
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

        swap.used_kb += match values[3].parse::<f64>() {
            Ok(r) => (r * 1.024) as u64,
            Err(_) => 0 as u64,
        };
        swap.total_kb += match values[2].parse::<f64>() {
            Ok(r) => (r * 1.024) as u64,
            Err(_) => 0 as u64,
        };
    }

    if swap.total_kb != 0 {
        swap.percent = (swap.used_kb as f32 / swap.total_kb as f32) * 100.0;
    }

    Ok(swap)
}
