use core::str;
use std::env;

use serde::Deserialize;
use x11rb::{connection::RequestConnection, protocol::{randr::{self, MonitorInfo}, xproto::{ConnectionExt, CreateWindowAux, Screen, Window, WindowClass}}, COPY_DEPTH_FROM_PARENT};

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    width: u16,
    height: u16,
    refresh_rate: Option<u16>
}
#[derive(Deserialize)]
pub struct DisplayConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Module for DisplayInfo {
    fn new() -> DisplayInfo {
        DisplayInfo {
            name: "".to_string(),
            width: 0,
            height: 0,
            refresh_rate: None
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.displays.title_color).is_some() {
            title_color = config.displays.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.displays.title_bold.is_some() {
            title_bold = config.displays.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.displays.title_italic.is_some() {
            title_italic = config.displays.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.displays.seperator.is_some() {
            seperator = config.displays.seperator.as_ref().unwrap();
        }

        let mut title: String = config.displays.title.clone();
        title = title.replace("{name}", &self.name);

        self.default_style(config, max_title_size, &title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let refresh_rate: String = match self.refresh_rate {
            Some(r) => r.to_string(),
            None => "N/A".to_string(),
        };
        config.displays.format.replace("{name}", &self.name)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &refresh_rate)
    }
}

pub fn get_displays() -> Result<Vec<DisplayInfo>, ModuleError> {
    // Good news, during my college final deadline hell over the past 2 months, I learned how to
    // use a display server connection!
    
    // Instead of relying on XDG_SESSION_TYPE line Desktop, I simply just check the sockets as it
    // can report any string and break if someone's dumb enough to do that
    if env::var("WAYLAND_DISPLAY").is_ok() {
        todo!();
    } else if env::var("DISPLAY").is_ok() {
        return fetch_xorg();
    } else {
        return Err(ModuleError::new("Display", format!("Could not identify desktop session type.")));
    }
}

fn fetch_xorg() -> Result<Vec<DisplayInfo>, ModuleError> {
    // This has really opened my eyes as to why more pieces of software haven't swapped over to
    // Wayland yet, it's so much more convoluted at times compared to X11
    let (conn, screen_num) = match x11rb::connect(None) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Display", format!("Can't connect to X11 server: {}", e))),
    };

    let screen: &Screen = &x11rb::connection::Connection::setup(&conn).roots[screen_num];
    let win_id: Window = match x11rb::connection::Connection::generate_id(&conn) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Display", format!("Can't create new X11 identifier: {}", e))),
    };
    match conn.create_window(COPY_DEPTH_FROM_PARENT, win_id, screen.root,
        0, 0,
        1, 1,
        0,
        WindowClass::INPUT_OUTPUT, 0, &CreateWindowAux::new()) 
    {
        Ok(_) => {},
        Err(e) => return Err(ModuleError::new("Display", format!("Can't create new X11 window: {}", e))),
    };

    if !conn.extension_information(randr::X11_EXTENSION_NAME).is_ok() {
        return Err(ModuleError::new("Display", "X11 compositor doesn't have required 'randr' extension.".to_string()));
    }

    let monitors: Vec<MonitorInfo> = match randr::get_monitors(&conn, win_id, true) {
        Ok(r) => match r.reply() {
            Ok(r) => r.monitors,
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {}", e))),
        },
        Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {}", e))),
    };
    let mut displays: Vec<DisplayInfo> = Vec::new();
    for monitor in monitors {
        let display = DisplayInfo {
            name: monitor.name.to_string(),
            width: monitor.width,
            height: monitor.height,
            refresh_rate: None, // Can't get on X11, or at least if you can I don't know how
        };
        displays.push(display);
    }

    // Not error checked as it's not a huge problem if this fails, moreso just a "would be nice to do" thing
    let _ = conn.destroy_window(win_id);
    Ok(displays)
}
