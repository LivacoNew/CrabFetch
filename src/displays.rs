use core::str;
use std::{fs::{read_dir, File, ReadDir}, io::Read, path::PathBuf, str::Split};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, Module, CONFIG};

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    width: u64,
    height: u64,
    refresh_rate: u32
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
            refresh_rate: 0
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.displays.title_color).is_some() {
            title_color = &CONFIG.displays.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.displays.title_bold).is_some() {
            title_bold = CONFIG.displays.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.displays.title_italic).is_some() {
            title_italic = CONFIG.displays.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.displays.seperator.is_some() {
            seperator = CONFIG.displays.seperator.as_ref().unwrap();
        }

        let mut title: String = CONFIG.displays.title.clone();
        title = title.replace("{name}", &self.name);

        self.default_style(&title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.displays.format.replace("{name}", &self.name)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &self.refresh_rate.to_string())
    }
}

pub fn get_displays() -> Vec<DisplayInfo> {
    let mut displays: Vec<DisplayInfo> = Vec::new();

    // (Thanks to FastFetch's SC for hinting the existance of edid to me and https://www.extron.com/article/uedid for a actual explanation for what it is)
    // - Find all /sys/class/drm/card-* folders
    // - Check for a "enabled" file and ensure it reads "enabled"
    // - "status" file gives if it's connected or not
    // - if it is connected, go into "edid" and parse it

    // Find all /sys/class/drm/ folders and scan for any that read card*-*
    let dir: ReadDir = match read_dir("/sys/class/drm") {
        Ok(r) => r,
        Err(_) => {
            return displays
        },
    };
    for d in dir {
        let mut display: DisplayInfo = DisplayInfo::new();

        if d.is_err() {
            continue
        }
        let path: PathBuf = d.unwrap().path();
        let file_name: &str = match path.file_name() {
            Some(r) => r.to_str().unwrap(),
            None => "",
        };
        if !file_name.starts_with("card") || !file_name.contains('-') || file_name.contains("Writeback") {
            continue
        }

        // We've confirmed it's a valid display, now we check it's enabled
        let enabled_path: PathBuf = path.join("enabled");
        if !enabled_path.exists() {
            continue
        }
        let mut enabled_file: File = match File::open(enabled_path) {
            Ok(f) => f,
            Err(_) => {
                continue
            },
        };
        let mut contents: String = String::new();
        match enabled_file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(_) => {
                continue
            },
        }
        if contents.trim() != "enabled" {
            continue
        }

        // Display Name
        let mut file_name_split: Split<'_, &str> = file_name.split("-");
        file_name_split.next();
        display.name = file_name_split.collect::<Vec<&str>>().join("-");

        // And now the hard part; edid


        displays.push(display);
    }

    displays
}
