use chrono::{DateTime, Local};
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, module::Module};

pub struct DateTimeInfo {
    datetime: DateTime<Local>,
}
#[derive(Deserialize)]
pub struct DateTimeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
}
impl Module for DateTimeInfo {
    fn new() -> DateTimeInfo {
        DateTimeInfo {
            datetime: Local::now(),
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.datetime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.datetime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.datetime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.datetime.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.datetime.format, config), config);

        Self::default_style(config, &config.datetime.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.datetime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.datetime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.datetime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.datetime.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, &config.datetime.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        self.datetime.format(text).to_string()
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on datetime module. This should never happen, please make a bug report!")
    }
}

pub fn get_date_time() -> DateTimeInfo {
    DateTimeInfo::new() // lol
}
