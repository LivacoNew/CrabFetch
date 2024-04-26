use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

pub struct BatteryInfo {
    percentage: u8,
}
#[derive(Deserialize)]
pub struct BatteryConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,

    pub path: String // Will default to BAT0
}
impl Default for BatteryConfiguration {
    fn default() -> Self {
        BatteryConfiguration {
            title: "Battery".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{percentage}%".to_string(),
            path: "BAT0".to_string()
        }
    }
}

impl Module for BatteryInfo {
    fn new() -> BatteryInfo {
        BatteryInfo {
            percentage: 0
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
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

        self.default_style(config, max_title_length, &config.battery.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.battery.format.replace("{percentage}", &self.percentage.to_string())
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
