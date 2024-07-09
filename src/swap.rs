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
        let title_color: &CrabFetchColor = config.swap.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.swap.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.swap.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.swap.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.swap.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.swap.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.swap.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.swap.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.swap.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.swap.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let dec_places: u32 = config.swap.decimal_places.unwrap_or(config.decimal_places);
        let use_ibis: bool = config.swap.use_ibis.unwrap_or(config.use_ibis);

        let mut bar: String = String::new();
        if config.swap.format.contains("{bar}") {
            let left_border: &str = config.swap.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.swap.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.swap.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.swap.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.swap.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percent, length);
        }

        formatter::process_percentage_placeholder(&config.swap.format, SwapInfo::round(self.percent, dec_places), config)
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
    let mut lines: Vec<&str> = contents.split('\n').collect();
    lines.remove(0);
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut values: Vec<&str> = line.split(['\t', ' ']).collect();
        values.retain(|x| x.trim() != "");

        swap.used_kb += match values[3].parse::<f64>() {
            Ok(r) => (r * 1.024) as u64,
            Err(_) => 0_u64,
        };
        swap.total_kb += match values[2].parse::<f64>() {
            Ok(r) => (r * 1.024) as u64,
            Err(_) => 0_u64,
        };
    }

    if swap.total_kb != 0 {
        swap.percent = (swap.used_kb as f32 / swap.total_kb as f32) * 100.0;
    }

    Ok(swap)
}
