use std::env;

#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util::{self, is_flag_set_u32}, ModuleError};

pub struct DesktopInfo {
    desktop: String,
    display_type: String
}
#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
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

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.desktop.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.desktop.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.desktop.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.desktop.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.desktop.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.desktop.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.desktop.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.desktop.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.desktop.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.desktop.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.desktop.title
            .replace("{desktop}", "Unknown")
            .replace("{display_type}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{desktop}", &self.desktop)
            .replace("{display_type}", &self.display_type)
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{desktop}") {
            info_flags |= DESKTOP_INFOFLAG_DESKTOP;
        }
        if format.contains("{display_type}") {
            info_flags |= DESKTOP_INFOFLAG_DISPLAY_TYPE;
        }

        info_flags
    }
}

const DESKTOP_INFOFLAG_DESKTOP: u32 = 1;
const DESKTOP_INFOFLAG_DISPLAY_TYPE: u32 = 2;

pub fn get_desktop(config: &Configuration) -> Result<DesktopInfo, ModuleError> {
    let mut desktop: DesktopInfo = DesktopInfo::new();
    let info_flags: u32 = DesktopInfo::gen_info_flags(&config.desktop.format);

    if util::in_wsl() {
        // WSLG weird shit https://github.com/microsoft/wslg
        desktop.desktop = "WSLG".to_string();
        desktop.display_type = "Wayland".to_string();
        return Ok(desktop);
    }

    if is_flag_set_u32(info_flags, DESKTOP_INFOFLAG_DESKTOP) {
        desktop.desktop = match env::var("XDG_CURRENT_DESKTOP") {
            Ok(r) => r,
                Err(_) => match env::var("DESKTOP_SESSION") {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("Desktop", format!("Could not parse $XDG_CURRENT_DESKTOP or $DESKTOP_SESSION env variable: {e}")))
            }
        };
    }

    if is_flag_set_u32(info_flags, DESKTOP_INFOFLAG_DISPLAY_TYPE) {
        desktop.display_type = if env::var("WAYLAND_DISPLAY").is_ok() {
            "wayland".to_string()
        } else if env::var("DISPLAY").is_ok() {
            "x11".to_string()
        } else {
            match env::var("XDG_SESSION_TYPE") {
                Ok(r) => r,
                Err(_) => return Err(ModuleError::new("Desktop", "Could not identify desktop session type.".to_string()))
            }
        }
    }

    Ok(desktop)
}
