use std::{fmt::Display, env};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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
impl Module for DesktopInfo {
    fn new() -> DesktopInfo {
        DesktopInfo {
            desktop: "".to_string(),
            display_type: "".to_string()
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.desktop.title_color).is_some() {
            title_color = &CONFIG.desktop.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.desktop.title_bold).is_some() {
            title_bold = CONFIG.desktop.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.desktop.title_italic).is_some() {
            title_italic = CONFIG.desktop.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.desktop.seperator.is_some() {
            seperator = CONFIG.desktop.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.desktop.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.desktop.format.replace("{desktop}", &self.desktop)
            .replace("{display_type}", &self.display_type)
    }
}
impl Display for DesktopInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.desktop, self.display_type)
    }
}

pub fn get_desktop() -> DesktopInfo {
    let mut desktop: DesktopInfo = DesktopInfo::new();

    desktop.desktop = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(r) => r,
        Err(e) => {
            log_error("Desktop", format!("Could not parse $XDG_CURRENT_DESKTOP env variable: {}", e));
            "Unknown".to_string()
        }
    };

    desktop.display_type = match env::var("XDG_SESSION_TYPE") {
        Ok(r) => r,
        Err(e) => {
            log_error("Desktop", format!("Could not parse $XDG_SESSION_TYPE env variable: {}", e));
            "Unknown".to_string()
        }
    };

    desktop
}
