use std::env;

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, module::Module, ModuleError};

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
    pub separator: Option<String>,
    pub format: String,
}
impl Module for DesktopInfo {
    fn new() -> DesktopInfo {
        DesktopInfo {
            desktop: "Unknown".to_string(),
            display_type: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.desktop.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.desktop.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.desktop.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.desktop.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.desktop.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.desktop.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.desktop.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.desktop.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.desktop.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.desktop.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.desktop.format.replace("{desktop}", &self.desktop)
            .replace("{display_type}", &self.display_type)
    }

    fn gen_info_flags(&self, format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{desktop}") {
            info_flags += DESKTOP_INFOFLAG_DESKTOP
        }
        if format.contains("{display_type}") {
            info_flags += DESKTOP_INFOFLAG_DISPLAY_TYPE
        }

        info_flags
    }
}

const DESKTOP_INFOFLAG_DESKTOP: u32 = 1;
const DESKTOP_INFOFLAG_DISPLAY_TYPE: u32 = 2;

pub fn get_desktop(config: &Configuration) -> Result<DesktopInfo, ModuleError> {
    let mut desktop: DesktopInfo = DesktopInfo::new();
    let info_flags: u32 = desktop.gen_info_flags(&config.desktop.format);

    if info_flags & DESKTOP_INFOFLAG_DESKTOP > 0 {
        desktop.desktop = match env::var("XDG_CURRENT_DESKTOP") {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Desktop", format!("Could not parse $XDG_CURRENT_DESKTOP env variable: {}", e)))
        };
    }

    if info_flags & DESKTOP_INFOFLAG_DISPLAY_TYPE > 0 {
        desktop.display_type = match env::var("XDG_SESSION_TYPE") {
            Ok(r) => r,
            Err(_) => {
                // Check if WAYLAND_DISPLAY is set 
                // If not, we'll check if DISPLAY is set 
                // Otherwise we have no idea
                if env::var("WAYLAND_DISPLAY").is_ok() {
                    "wayland".to_string()
                } else if env::var("DISPLAY").is_ok() {
                    "x11".to_string()
                } else {
                    return Err(ModuleError::new("Desktop", "Could not identify desktop session type.".to_string()));
                }
            }
        };
    }

    Ok(desktop)
}
