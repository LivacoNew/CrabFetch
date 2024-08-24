use std::fs::{read_dir, ReadDir};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, module::Module, ModuleError};

pub struct ProcessesInfo {
    count: u32 // god forbid someone manages to hit this limit
}
#[derive(Deserialize)]
pub struct ProcessesConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: Option<String>,
}
impl Module for ProcessesInfo {
    fn new() -> ProcessesInfo {
        ProcessesInfo {
            count: 0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.processes.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.processes.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.processes.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.processes.separator.as_ref().unwrap_or(&config.separator);

        let format: String = config.processes.format.clone().unwrap_or("{count}".to_string());
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&format, config));

        Self::default_style(config, max_title_size, &config.processes.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.processes.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.processes.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.processes.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.processes.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.processes.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{count}", &self.count.to_string())
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on processes module. This should never happen, please make a bug report!")
    }
}

pub fn get_process_count() -> Result<ProcessesInfo, ModuleError> {
    let mut process_info: ProcessesInfo = ProcessesInfo::new();

    // Scans /proc and simply checks if it's a number 
    let dir: ReadDir = match read_dir("/proc") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Processes", format!("Failed to read /proc: {}", e)))
    };

    for x in dir {
        let x = x.unwrap().file_name().into_string().unwrap();
        if x.parse::<u64>().is_ok() {
            process_info.count += 1;
        }
    }

    Ok(process_info)
}
