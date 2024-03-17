use core::str;
use std::{fmt::Display, env};

use crate::Module;

pub struct DesktopInfo {
    desktop: String,
}
impl Module for DesktopInfo {
    fn new() -> DesktopInfo {
        DesktopInfo {
            desktop: "".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{desktop}", &self.desktop)
    }
}
impl Display for DesktopInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.desktop)
    }
}

pub fn get_desktop() -> DesktopInfo {
    let mut desktop = DesktopInfo::new();
    get_basic_info(&mut desktop);

    desktop
}

fn get_basic_info(desktop: &mut DesktopInfo) {
    // Gets the username from $USER
    // Gets the hostname from /etc/hostname
    desktop.desktop = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(r) => r,
        Err(e) => {
            println!("WARNING: Could not parse $XDG_CURRENT_DESKTOP env variable: {}", e);
            "Unknown".to_string()
        }
    };
}
