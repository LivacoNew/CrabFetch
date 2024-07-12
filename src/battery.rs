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
        let title_color: &CrabFetchColor = config.battery.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.battery.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.battery.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.battery.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.battery.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.battery.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.battery.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.battery.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.battery.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.battery.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let dec_places: u32 = config.battery.decimal_places.unwrap_or(config.decimal_places);

        let mut bar: String = String::new();
        if config.battery.format.contains("{bar}") {
            let left_border: &str = config.battery.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.battery.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.battery.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.battery.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.battery.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percentage, length);
        }

        formatter::process_percentage_placeholder(&config.battery.format, BatteryInfo::round(self.percentage, dec_places), config)
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
