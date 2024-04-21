use std::{fs::File, io::Read, time::{Duration, Instant}};

use humantime::format_duration;
use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.uptime.title_color).is_some() {
            title_color = &CONFIG.uptime.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if CONFIG.uptime.title_bold.is_some() {
            title_bold = CONFIG.uptime.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if CONFIG.uptime.title_italic.is_some() {
            title_italic = CONFIG.uptime.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.uptime.seperator.is_some() {
            seperator = CONFIG.uptime.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.uptime.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.uptime.format.replace("{time}", &format_duration(self.uptime).to_string())
    }
}

pub fn get_uptime() -> UptimeInfo {
    let mut uptime: UptimeInfo = UptimeInfo::new();

    // Grabs from /proc/uptime
    let mut file: File = match File::open("/proc/uptime") {
        Ok(r) => r,
        Err(e) => {
            log_error("Uptime", format!("Can't read from /proc/uptime - {}", e));
            return uptime
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Uptime", format!("Can't read from /proc/uptime - {}", e));
            return uptime
        },
    }
    uptime.uptime = match contents.split(" ").collect::<Vec<&str>>()[0].parse::<f64>() {
        Ok(r) => Duration::new(r.floor() as u64, 0),
        Err(e) => {
            log_error("Uptime", format!("Could not parse /proc/uptime: {}", e));
            Duration::new(0, 0)
        },
    };

    uptime
}
