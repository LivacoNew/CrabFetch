use core::str;
use std::{fs::{self, read_dir, File, ReadDir}, io::Read, path::PathBuf, str::Split};

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
    // And to address the elephant in the room, yes this is a cheap and not technically correct way to do this. Unfortunately I don't have the knowledge, time nor patience to write a display server connection just for some resolution details.

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

        // And now the hard part; edid - If this is wrong let me know and point me in the right
        // direction, as I've never worked with this before
        // HUGE thanks to this lovely document https://glenwing.github.io/docs/VESA-EEDID-A2.pdf
        // The only disadvantage here is that we can't get the *current* resolution, only the max
        // res but I'm fine with that

        let edid_bytes: Vec<u8> = match fs::read(path.join("edid")) {
            Ok(r) => r,
            Err(_) => {
                continue
            },
        };
        if edid_bytes.len() == 0 {
            // This can happen in VM's, meaning no display output. Cus of this, I just push the
            // display empty
            displays.push(display);
            continue
        }

        // DTD starts at byte 54
        // Formula thanks to https://stackoverflow.com/a/10299885 and https://stackoverflow.com/a/4476144
        let resolution_w: u32 = (u32::from(edid_bytes[58]) >> 4) << 8 | u32::from(edid_bytes[56]);
        let resolution_h: u32 = (u32::from(edid_bytes[61]) >> 4) << 8 | u32::from(edid_bytes[59]);
        display.width = resolution_w as u64;
        display.height = resolution_h as u64;

        // Refresh rate now, this is grabbed from the Pixel Clock
        // Credit for the formula: https://electronics.stackexchange.com/a/492180
        // let mut pixel_clock: u64 = (u64::from(edid_bytes[54]) | u64::from(edid_bytes[55]) << 8) * 10000;
        // let blanking_w: u32 = u32::from(edid_bytes[57]) | (u32::from(edid_bytes[58]) & 0b00001111) << 8;
        // let blanking_h: u32 = u32::from(edid_bytes[60]) | (u32::from(edid_bytes[61]) & 0b00001111) << 8;

        // Starts at byte 128
        // Let's try the display range limits now which should give us the absolute maximum rate
        // let is_descriptor: u16 = (u16::from(edid_bytes[128]) << 8) & u16::from(edid_bytes[128+1]);
        // if is_descriptor == 0 {
        //     let range_limits_only: u8 = (edid_bytes[128+10]);
        //     pixel_clock = (u64::from(edid_bytes[81])) * 10000000;
        // }

        // let total_pixels: u64 = (resolution_w as u64 + blanking_w as u64) * (resolution_h as u64 + blanking_h as u64);
        // let refresh_rate: u32 = (pixel_clock / total_pixels) as u32;
        // display.refresh_rate = refresh_rate;

        displays.push(display);
    }

    displays
}
