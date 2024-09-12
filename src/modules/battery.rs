use std::{fs::{self, DirEntry, ReadDir}, path::PathBuf};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, util, ModuleError};

pub struct BatteryInfo {
    model_name: String,
    percentage: f32,
}
#[derive(Deserialize)]
pub struct BatteryConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub progress_left_border: Option<String>,
    pub progress_right_border: Option<String>,
    pub progress_progress: Option<String>,
    pub progress_empty: Option<String>,
    pub progress_target_length: Option<u8>,
    pub decimal_places: Option<u32>,
}
impl Module for BatteryInfo {
    fn new() -> BatteryInfo {
        BatteryInfo {
            model_name: "Unknown".to_string(),
            percentage: 0.0
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.battery.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.battery.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.battery.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.battery.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.battery.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.battery.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.battery.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.battery.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.battery.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.battery.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.battery.title
            .replace("{model_name}", "Unknown").to_string()
            .replace("{percentage}", "Unknown").to_string()
            .replace("{bar}", "").to_string();

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let dec_places: u32 = config.battery.decimal_places.unwrap_or(config.decimal_places);

        let mut bar: String = String::new();
        if text.contains("{bar}") {
            let left_border: &str = config.battery.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.battery.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.battery.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.battery.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.battery.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percentage, length);
        }

        formatter::process_percentage_placeholder(text, formatter::round(self.percentage as f64, dec_places) as f32, config)
            .replace("{model_name}", &self.model_name)
            .replace("{percentage}", &self.percentage.to_string())
            .replace("{bar}", &bar)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on battery module. This should never happen, please make a bug report!")
    }
}

pub fn get_batteries() -> Result<Vec<BatteryInfo>, ModuleError> {
    let mut batteries: Vec<BatteryInfo> = Vec::new();

    let dir: ReadDir = match fs::read_dir("/sys/class/power_supply/") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Battery", format!("Can't read from /sys/class/power_supply: {}", e))),
    };
    for d in dir {
        if d.is_err() {
            continue;
        }
        let d: DirEntry = d.unwrap();
        let path: PathBuf = d.path();
        // From what I can tell, this dir is only batteries so parsing is easy
        let percentage: f32 = match util::file_read(&path.join("capacity")) {
            Ok(r) => {
                match r.trim().parse() {
                    Ok(r) => r,
                    Err(_) => continue,
                    // Err(e) => return Err(ModuleError::new("Battery", format!("Can't parse value from {} - {}", path, e))),
                }
            },
            // Err(e) => return Err(ModuleError::new("Battery", format!("Can't read from {} - {}", path, e))),
            Err(_) => continue,
        };
        let model_name: String = match util::file_read(&path.join("model_name")) {
            Ok(r) => r.trim().to_string(),
            Err(_) => match path.file_name() {
                Some(r) => r.to_str().unwrap_or("Unknown").to_string(),
                None => continue,
            },
        };

        batteries.push(BatteryInfo {
            model_name,
            percentage
        })
    }


    Ok(batteries)
}
