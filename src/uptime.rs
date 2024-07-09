use std::{fs::File, io::Read, time::Duration};

use humantime::format_duration;
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

pub struct UptimeInfo {
    uptime: Duration,
}
#[derive(Deserialize)]
pub struct UptimeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Module for UptimeInfo {
    fn new() -> UptimeInfo {
        UptimeInfo {
            uptime: Duration::new(0, 0),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.uptime.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.uptime.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.uptime.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.uptime.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.uptime.format.replace("{time}", &format_duration(self.uptime).to_string())
    }
}

pub fn get_uptime() -> Result<UptimeInfo, ModuleError> {
    let mut uptime: UptimeInfo = UptimeInfo::new();

    // Grabs from /proc/uptime
    let mut file: File = match File::open("/proc/uptime") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Uptime", format!("Can't read from /proc/uptime - {}", e))),
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => return Err(ModuleError::new("Uptime", format!("Can't read from /proc/uptime - {}", e))),
    }
    uptime.uptime = match contents.split(' ').collect::<Vec<&str>>()[0].parse::<f64>() {
        Ok(r) => Duration::new(r.floor() as u64, 0),
        Err(e) => return Err(ModuleError::new("Uptime", format!("Could not parse /proc/uptime: {}", e))),
    };

    Ok(uptime)
}
