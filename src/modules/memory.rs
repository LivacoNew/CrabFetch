use std::fs::File;
use std::io::{BufRead, BufReader};

use schemars::JsonSchema;
use serde::Deserialize;

use crate::{formatter::{self, CrabFetchColor}, config_manager::Configuration, module::Module, ModuleError};

pub struct MemoryInfo {
    used_kb: u64,
    max_kb: u64,
    percentage: f32
}
#[derive(Deserialize, JsonSchema)]
pub struct MemoryConfiguration {
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
    pub use_ibis: Option<bool>,
    pub decimal_places: Option<u32>
}
impl Module for MemoryInfo {
    fn new() -> MemoryInfo {
        MemoryInfo {
            used_kb: 0,
            max_kb: 0,
            percentage: 0.0
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.memory.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.memory.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.memory.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.memory.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.memory.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.memory.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.memory.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.memory.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.memory.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.memory.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.memory.title
            .replace("{used}", "Unknown")
            .replace("{max}", "Unknown")
            .replace("{bar}", "")
            .replace("{percentage}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let dec_places: u32 = config.memory.decimal_places.unwrap_or(config.decimal_places);
        let use_ibis: bool = config.memory.use_ibis.unwrap_or(config.use_ibis);

        let mut bar: String = String::new();
        if text.contains("{bar}") {
            let left_border: &str = config.memory.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.memory.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.memory.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.memory.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.memory.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percentage, length);
        }

        #[allow(clippy::cast_possible_truncation)]
        formatter::process_percentage_placeholder(text, formatter::round(f64::from(self.percentage), dec_places) as f32, config)
            .replace("{used}", &formatter::auto_format_bytes(self.used_kb, use_ibis, dec_places))
            .replace("{max}", &formatter::auto_format_bytes(self.max_kb, use_ibis, dec_places))
            .replace("{bar}", &bar.to_string())
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on memory module. This should never happen, please make a bug report!")
    }
}

// Clippy had a lot of issues with this function, while it's kinda not possible to improve it
// without just being a pain in the ass to work with
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn get_memory() -> Result<MemoryInfo, ModuleError> {
    // no info flags here as while it would've had a slight benefit, all the info requires eachother anyway so
    // it's hardly worth it
    let mut memory: MemoryInfo = MemoryInfo::new();

    // Fetches from /proc/meminfo
    let file: File = match File::open("/proc/meminfo") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Memory", format!("Can't read from /proc/meminfo - {e}"))),
    };

    let mut mem_available: u64 = 0;
    let buffer: BufReader<File> = BufReader::new(file);
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.starts_with("MemTotal") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = var[..var.len() - 3].trim();
            memory.max_kb = match var.to_string().parse::<f64>() {
                Ok(r) => (r * 1.024) as u64,
                Err(e) => return Err(ModuleError::new("Memory", format!("Could not parse total memory: {e}")))
            }
        }
        if line.starts_with("MemAvailable") {
            let mut var: &str = line.split(": ").collect::<Vec<&str>>()[1];
            var = var[..var.len() - 3].trim();
            mem_available = match var.to_string().parse::<f64>() {
                Ok(r) => (r * 1.024) as u64,
                Err(e) => return Err(ModuleError::new("Memory", format!("Could not parse memfree memory: {e}")))
            }
        }
        if memory.max_kb != 0 && mem_available != 0 {
            break;
        }
    }

    memory.used_kb = memory.max_kb - mem_available;
    memory.percentage = (memory.used_kb as f32 / memory.max_kb as f32) * 100.0;

    Ok(memory)
}
