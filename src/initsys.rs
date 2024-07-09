use core::str;
use std::fs;

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

pub struct InitSystemInfo {
    initsys: String,
}
#[derive(Deserialize)]
pub struct InitSystemConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: Option<String>
}
impl Module for InitSystemInfo {
    fn new() -> InitSystemInfo {
        InitSystemInfo {
            initsys: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.initsys.title_color).is_some() {
            title_color = &config.initsys.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.initsys.title_bold.is_some() {
            title_bold = config.initsys.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.initsys.title_italic.is_some() {
            title_italic = config.initsys.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.initsys.seperator.is_some() {
            seperator = config.initsys.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.initsys.title_color).is_some() {
            title_color = config.initsys.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.initsys.title_bold.is_some() {
            title_bold = config.initsys.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.initsys.title_italic.is_some() {
            title_italic = config.initsys.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.initsys.seperator.is_some() {
            seperator = config.initsys.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut format: String = "{initsys}".to_string();
        if config.initsys.format.is_some() {
            format = config.initsys.format.clone().unwrap();
        }

        format.replace("{initsys}", &self.initsys)
    }
}

pub fn get_init_system() -> Result<InitSystemInfo, ModuleError> {
    let mut initsys: InitSystemInfo = InitSystemInfo::new();

    // Just gets the symlink from /sbin/init 
    initsys.initsys = match fs::canonicalize("/sbin/init") {
        Ok(r) => r.display().to_string().split("/").last().unwrap().to_string(),
        Err(e) => return Err(ModuleError::new("InitSys", format!("Failed to canonicalize /sbin/init symlink: {}", e)))
    };

    Ok(initsys)
}
