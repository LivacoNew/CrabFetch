use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::{self, Configuration, CrabFetchColor, ModuleConfiguration, TOMLParseError}, Module, ModuleError};

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
impl ModuleConfiguration for BatteryConfiguration {
    fn apply_toml_line(&mut self, key: &str, value: &str) -> Result<(), crate::config_manager::TOMLParseError> {
        match key {
            "title" => self.title = config_manager::toml_parse_string(value)?,
            "title_color" => self.title_color = Some(config_manager::toml_parse_string_to_color(value)?),
            "title_bold" => self.title_bold = Some(config_manager::toml_parse_bool(value)?),
            "title_italic" => self.title_italic = Some(config_manager::toml_parse_bool(value)?),
            "seperator" => self.seperator = Some(config_manager::toml_parse_string(value)?),
            "format" => self.format = config_manager::toml_parse_string(value)?,
            "path" => self.format = config_manager::toml_parse_string(value)?,
            _ => return Err(TOMLParseError::new("Unknown key.".to_string(), Some("Battery".to_string()), Some(key.to_string()), value.to_string()))
        }
        Ok(())
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
