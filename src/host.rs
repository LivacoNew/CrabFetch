use core::str;
use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;

use crate::{colors::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

pub struct HostInfo {
    host: String,
}
#[derive(Deserialize)]
pub struct HostConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: Option<String>
}
impl Module for HostInfo {
    fn new() -> HostInfo {
        HostInfo {
            host: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.host.title_color).is_some() {
            title_color = &config.host.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.host.title_bold.is_some() {
            title_bold = config.host.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.host.title_italic.is_some() {
            title_italic = config.host.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.host.seperator.is_some() {
            seperator = config.host.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.host.title_color).is_some() {
            title_color = config.host.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.host.title_bold.is_some() {
            title_bold = config.host.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.host.title_italic.is_some() {
            title_italic = config.host.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.host.seperator.is_some() {
            seperator = config.host.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.host.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut format: String = "{host}".to_string();
        if config.host.format.is_some() {
            format = config.host.format.clone().unwrap();
        }

        format.replace("{host}", &self.host)
    }
}

pub fn get_host() -> Result<HostInfo, ModuleError> {
    let mut host: HostInfo = HostInfo::new();

    // Prioritises product_name for laptops, then goes to board_name
    let mut chosen_path: Option<&str> = None;
    if Path::new("/sys/devices/virtual/dmi/id/product_name").exists() {
        chosen_path = Some("/sys/devices/virtual/dmi/id/product_name");
    } else if Path::new("/sys/devices/virtual/dmi/id/board_name").exists() {
        chosen_path = Some("/sys/devices/virtual/dmi/id/board_name");
    }
    if chosen_path.is_none() {
        return Err(ModuleError::new("Host", "Can't find an appropriate path for host.".to_string()));
    }
    let chosen_path: &str = chosen_path.unwrap();

    let mut file: File = match File::open(chosen_path) {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path, e)));
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(ModuleError::new("Host", format!("Can't read from {} - {}", chosen_path, e)));
        },
    }

    host.host = contents.trim().to_string();

    Ok(host)
}
