use core::str;
use std::{fmt::Display, env};

use crate::{log_error, Module};

pub struct DesktopInfo {
    desktop: String,
    display_type: String
}
impl Module for DesktopInfo {
    fn new() -> DesktopInfo {
        DesktopInfo {
            desktop: "".to_string(),
            display_type: "".to_string()
        }
    }

    fn style(&self) -> String {
        todo!()
    }

    fn replace_placeholders(&self) -> String {
        todo!()
    }
    // fn format(&self, format: &str, _: u32) -> String {
    //     format.replace("{desktop}", &self.desktop)
    //         .replace("{display_type}", &self.display_type)
    // }
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
