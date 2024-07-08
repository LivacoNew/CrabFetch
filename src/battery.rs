use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, Module, ModuleError};

pub struct BatteryInfo {
    percentage: f32,
}
#[derive(Deserialize)]
pub struct BatteryConfiguration {
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
    pub decimal_places: Option<u32>,
    pub path: String // Will default to BAT0
}
impl Module for BatteryInfo {
    fn new() -> BatteryInfo {
        BatteryInfo {
            percentage: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.battery.title_color).is_some() {
            title_color = &config.battery.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.battery.title_bold.is_some() {
            title_bold = config.battery.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.battery.title_italic.is_some() {
            title_italic = config.battery.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.battery.seperator.is_some() {
            seperator = config.battery.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.battery.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.battery.title_color).is_some() {
            title_color = config.battery.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.battery.title_bold.is_some() {
            title_bold = config.battery.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.battery.title_italic.is_some() {
            title_italic = config.battery.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.battery.seperator.is_some() {
            seperator = config.battery.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.battery.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut dec_places: u32 = config.decimal_places;
        if config.mounts.decimal_places.is_some() {
            dec_places = config.mounts.decimal_places.unwrap();
        }

        let mut bar: String = String::new();
        if config.battery.format.contains("{bar}") {
            let mut left_border: &str = config.progress_left_border.as_str();
            if config.battery.progress_left_border.is_some() {
                left_border = config.battery.progress_left_border.as_ref().unwrap();
            }
            let mut right_border: &str = config.progress_right_border.as_str();
            if config.battery.progress_right_border.is_some() {
                right_border = config.battery.progress_right_border.as_ref().unwrap();
            }
            let mut progress: &str = config.progress_progress.as_str();
            if config.battery.progress_progress.is_some() {
                progress = config.battery.progress_progress.as_ref().unwrap();
            }
            let mut empty: &str = config.progress_empty.as_str();
            if config.battery.progress_empty.is_some() {
                empty = config.battery.progress_empty.as_ref().unwrap();
            }
            let mut length: u8 = config.progress_target_length;
            if config.battery.progress_target_length.is_some() {
                length = config.battery.progress_target_length.unwrap();
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

        formatter::process_percentage_placeholder(&config.battery.format, BatteryInfo::round(self.percentage, dec_places), &config)
            .replace("{bar}", &bar)
    }
}

pub fn get_battery(battery_path: &str) -> Result<BatteryInfo, ModuleError> {
    let mut battery: BatteryInfo = BatteryInfo::new();

    // /sys/class/power_supply/{path}/capacity
    let path: String = format!("/sys/class/power_supply/{}/capacity", battery_path).to_string();
    let mut file: File = match File::open(&path) {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Battery", format!("Can't read from {} - {}", path, e)));
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("Battery", format!("Can't read from {} - {}", path, e)));
        },
    }

    battery.percentage = match contents.trim().parse() {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Battery", format!("Can't parse value from {} - {}", path, e)));
        },
    };

    Ok(battery)
}
