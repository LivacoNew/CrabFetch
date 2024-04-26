use std::env;

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

pub struct DesktopInfo {
    desktop: String,
    display_type: String
}
#[derive(Deserialize)]
pub struct DesktopConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Default for DesktopConfiguration {
    fn default() -> Self {
        DesktopConfiguration {
            title: "Desktop".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{desktop} ({display_type})".to_string()
        }
    }
}

impl Module for DesktopInfo {
    fn new() -> DesktopInfo {
        DesktopInfo {
            desktop: "".to_string(),
            display_type: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.desktop.title_color).is_some() {
            title_color = &config.desktop.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.desktop.title_bold.is_some() {
            title_bold = config.desktop.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.desktop.title_italic.is_some() {
            title_italic = config.desktop.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.desktop.seperator.is_some() {
            seperator = config.desktop.seperator.as_ref().unwrap();
        }

        self.default_style(config, max_title_length, &config.desktop.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.desktop.format.replace("{desktop}", &self.desktop)
            .replace("{display_type}", &self.display_type)
    }
}

pub fn get_desktop() -> Result<DesktopInfo, ModuleError> {
    let mut desktop: DesktopInfo = DesktopInfo::new();

    desktop.desktop = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Desktop", format!("Could not parse $XDG_CURRENT_DESKTOP env variable: {}", e)));
        }
    };

    desktop.display_type = match env::var("XDG_SESSION_TYPE") {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("Desktop", format!("Could not parse $XDG_SESSION_TYPE env variable: {}", e)));
        }
    };

    Ok(desktop)
}
