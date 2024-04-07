use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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
impl Module for BatteryInfo {
    fn new() -> BatteryInfo {
        BatteryInfo {
            percentage: 0
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.battery.title_color).is_some() {
            title_color = &CONFIG.battery.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.battery.title_bold).is_some() {
            title_bold = CONFIG.battery.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.battery.title_italic).is_some() {
            title_italic = CONFIG.battery.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.battery.seperator.is_some() {
            seperator = CONFIG.battery.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.battery.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.battery.format.replace("{percentage}", &self.percentage.to_string())
    }
}

pub fn get_battery() -> BatteryInfo {
    let mut battery: BatteryInfo = BatteryInfo::new();

    // /sys/class/power_supply/{path}/capacity
    let path: String = format!("/sys/class/power_supply/{}/capacity", CONFIG.battery.path).to_string();
    let mut file: File = match File::open(&path) {
        Ok(r) => r,
        Err(e) => {
            log_error("Battery", format!("Can't read from {} - {}", path, e));
            return battery
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Battery", format!("Can't read from {} - {}", path, e));
            return battery
        },
    }

    battery.percentage = match contents.trim().parse() {
        Ok(r) => r,
        Err(e) => {
            log_error("Battery", format!("Can't parse value from {} - {}", path, e));
            return battery
        },
    };

    battery
}
