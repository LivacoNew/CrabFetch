use core::str;
use std::{env, process::Command, io::ErrorKind::NotFound};

use serde::Deserialize;
use serde_json::Value;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

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

    let desktop = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Could not parse $XDG_CURRENT_DESKTOP env variable: {}", e));
            "Unknown".to_string()
        }
    };
    // KDE support
    if desktop == "KDE" {
        displays = match parse_kscreen_doctor() {
            Some(r) => r,
            None => Vec::new(),
        };
    } else {
        // Check if we're in x11 or wayland
        let session_type: Option<String> = match env::var("XDG_SESSION_TYPE") {
            Ok(r) => Some(r),
            Err(e) => {
                log_error("Displays", format!("Could not parse $XDG_SESSION_TYPE env variable: {}", e));
                None
            }
        };

        match session_type {
            Some(r) => {
                match r.as_str() {
                    "x11" => {
                        displays = match parse_xrandr() {
                            Some(r) => r,
                            None => Vec::new(),
                        };
                    }
                    "wayland" => {
                        // TODO: This only works on wlroots
                        displays = match parse_wlr_randr() {
                            Some(r) => r,
                            None => Vec::new(),
                        };
                    }
                    _ => {
                        log_error("Displays", format!("Unknown display server."));
                        return displays
                    }
                }
            },
            None => {
                log_error("Displays", format!("Unknown display server."));
                return displays
            },
        }
    }

    displays
}

fn parse_xrandr() -> Option<Vec<DisplayInfo>> {
    let output: Vec<u8> = match Command::new("xrandr")
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    log_error("Displays", format!("Display on x11 requires the 'xrandr' command, which is not present!"));
                } else {
                    log_error("Displays", format!("Unknown error while fetching x11 displays: {}", e));
                }

                return None
            },
        };
    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Unknown error while fetching x11 displays: {}", e));
            return None
        },
    };

    let mut result: Vec<DisplayInfo> = Vec::new();

    // This is really fuckin annoying to parse
    let mut last_display_index: usize = 0;
    for line in contents.split("\n") {
        if !line.contains("connected") {
            if !line.contains("*") {
                continue
            }
            if last_display_index == 0 {
                continue // oops
            }

            // Likely our last display's mode, meaning we get the refresh rate here
            let mut mode: Vec<&str> = line.split(" ").collect();
            mode.retain(|x| x.trim() != "");
            let mut rate: String = mode[1].to_string();
            rate = rate.replace("*", "");
            rate = rate.replace("+", "");

            result[last_display_index - 1].refresh_rate = rate.parse::<f32>().unwrap().round() as u32;
            continue
        }

        let values: Vec<&str> = line.split(" ").collect();
        let mut display = DisplayInfo::new();

        // Resolution
        // let resolution_str_full: &str = values[2];
        let resolution_str: Vec<&str> = values[2][0..values[2].find("+").unwrap()].split("x").collect();
        display.width = resolution_str[0].parse().unwrap();
        display.height = resolution_str[1].parse().unwrap();

        // Name
        display.name = values[0].to_string();

        result.push(display);
        last_display_index = result.len();
    }

    Some(result)
}

fn parse_wlr_randr() -> Option<Vec<DisplayInfo>> {
    let output: Vec<u8> = match Command::new("wlr-randr")
        .arg("--json") // GOD BLESS
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    log_error("Displays", format!("Display on wlroots requires the 'wlr-randr' command, which is not present!"));
                } else {
                    log_error("Displays", format!("Unknown error while fetching wlroots displays: {}", e));
                }

                return None
            },
        };
    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Unknown error while fetching wlroots displays: {}", e));
            return None
        },
    };

    let mut result: Vec<DisplayInfo> = Vec::new();

    let parsed: Vec<Value> = match serde_json::from_str(&contents) {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Unknown error while fetching wlroots displays: {}", e));
            return None
        },
    };

    for entry in parsed {
        // threw error checking out the window here, fuck that
        let mut display = DisplayInfo::new();

        // Resolution
        let modes: &Vec<Value> = entry["modes"].as_array().unwrap();
        for mode in modes {
            if !mode["current"].as_bool().unwrap() {
                continue
            }

            display.width = mode["width"].as_u64().unwrap();
            display.height = mode["height"].as_u64().unwrap();
            display.refresh_rate = mode["refresh"].as_f64().unwrap().round() as u32; // also stinky
        }

        // Name
        display.name = entry["name"].as_str().unwrap().to_string();

        result.push(display);
    }

    Some(result)
}

fn parse_kscreen_doctor() -> Option<Vec<DisplayInfo>> {
    let output: Vec<u8> = match Command::new("kscreen-doctor")
        .arg("--json")
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    log_error("Displays", format!("Display on KDE requires the 'kscreen-doctor' command, which is not present!"));
                } else {
                    log_error("Displays", format!("Unknown error while fetching KDE displays: {}", e));
                }

                return None
            },
        };
    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Unknown error while fetching KDE displays: {}", e));
            return None
        },
    };

    let mut result: Vec<DisplayInfo> = Vec::new();

    let parsed: Value = match serde_json::from_str(&contents) {
        Ok(r) => r,
        Err(e) => {
            log_error("Displays", format!("Unknown error while fetching KDE displays: {}", e));
            return None
        },
    };


    let outputs: &Vec<Value> = &parsed["outputs"].as_array().unwrap();
    for output in outputs {
        let mut display = DisplayInfo::new();
        if !output["enabled"].as_bool().unwrap() {
            continue
        }

        // Name
        display.name = (&output["name"]).as_str().unwrap().to_string();

        let current_mode: &str = output["currentModeId"].as_str().unwrap();
        for mode in output["modes"].as_array().unwrap() {
            if mode["id"] != current_mode {
                continue
            }

            // Resolution
            let size: &Value = &mode["size"];
            display.width = size["width"].as_u64().unwrap();
            display.height = size["height"].as_u64().unwrap();

            // Refresh Rate
            display.refresh_rate = mode["refreshRate"].as_f64().unwrap().round() as u32;
        }

        result.push(display);
    }

    Some(result)
}
