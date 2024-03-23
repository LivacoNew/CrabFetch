use core::str;
use std::{env, fmt::Display, process::Command, io::ErrorKind::NotFound};

use serde_json::Value;

use crate::Module;

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    width: u64,
    height: u64,
    refresh_rate: u32
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
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{name}", &self.name)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &self.refresh_rate.to_string())
    }
}
impl Display for DisplayInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}x{} @ {}hz", self.name, self.width, self.height, self.refresh_rate)
    }
}

pub fn get_displays() -> Vec<DisplayInfo> {
    let mut displays: Vec<DisplayInfo> = Vec::new();

    // Check if we're in x11 or wayland
    let session_type: Option<String> = match env::var("XDG_SESSION_TYPE") {
        Ok(r) => Some(r),
        Err(e) => {
            print!("Could not parse $XDG_SESSION_TYPE env variable: {}", e);
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
                    // Currently I only know of wlr-randr however I am aware there's no standard
                    // randr tool here
                    displays = match parse_wlr_randr() {
                        Some(r) => r,
                        None => Vec::new(),
                    };
                }
                _ => {
                    print!("Unknown display server.");
                    return displays
                }
            }
        },
        None => {
            print!("Unknown display server.");
            return displays
        },
    }

    displays
}

fn parse_xrandr() -> Option<Vec<DisplayInfo>> {
    let output: Vec<u8> = match Command::new("xrandr")
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    print!("Display on x11 requires the 'xrandr' command, which is not present!");
                } else {
                    print!("Unknown error while fetching x11 displays: {}", e);
                }

                return None
            },
        };
    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            print!("Unknown error while fetching x11 displays: {}", e);
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
                    print!("Display on wayland requires the 'wlr-randr' command, which is not present!");
                } else {
                    print!("Unknown error while fetching wayland displays: {}", e);
                }

                return None
            },
        };
    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            print!("Unknown error while fetching wayland displays: {}", e);
            return None
        },
    };

    let mut result: Vec<DisplayInfo> = Vec::new();

    let parsed: Vec<Value> = match serde_json::from_str(&contents) {
        Ok(r) => r,
        Err(e) => {
            print!("Unknown error while fetching wayland displays: {}", e);
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
