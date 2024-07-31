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
    pub separator: Option<String>,
    pub format: Option<String>
}
impl Module for InitSystemInfo {
    fn new() -> InitSystemInfo {
        InitSystemInfo {
            initsys: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let format: String = config.initsys.format.clone().unwrap_or("{initsys}".to_string());
        format.replace("{initsys}", &self.initsys)
    }
}

pub fn get_init_system() -> Result<InitSystemInfo, ModuleError> {
    let mut initsys: InitSystemInfo = InitSystemInfo::new();

    // Just gets the symlink from /sbin/init 
    initsys.initsys = match fs::canonicalize("/sbin/init") {
        Ok(r) => r.display().to_string()
            .split('/')
            .last()
            .unwrap()
            .to_string(),
        Err(e) => return Err(ModuleError::new("InitSys", format!("Failed to canonicalize /sbin/init symlink: {}", e)))
    };

    Ok(initsys)
}
