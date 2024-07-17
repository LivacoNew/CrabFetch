use chrono::{DateTime, Local};
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module};

pub struct DateTimeInfo {
    datetime: DateTime<Local>,
}
#[derive(Deserialize)]
pub struct DateTimeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Module for DateTimeInfo {
    fn new() -> DateTimeInfo {
        DateTimeInfo {
            datetime: Local::now(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.datetime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.datetime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.datetime.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.datetime.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.datetime.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.datetime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.datetime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.datetime.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.datetime.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.datetime.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        self.datetime.format(&config.datetime.format).to_string()
    }
}

pub fn get_date_time() -> DateTimeInfo {
    DateTimeInfo::new() // lol
}
