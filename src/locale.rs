use std::env;

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

pub struct LocaleInfo {
    code: String,
}
#[derive(Deserialize)]
pub struct LocaleConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String
}
impl Module for LocaleInfo {
    fn new() -> LocaleInfo {
        LocaleInfo {
            code: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.locale.title_color).is_some() {
            title_color = &config.locale.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.locale.title_bold.is_some() {
            title_bold = config.locale.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.locale.title_italic.is_some() {
            title_italic = config.locale.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.locale.seperator.is_some() {
            seperator = config.locale.seperator.as_ref().unwrap();
        }

        self.default_style(config, max_title_length, &config.locale.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.locale.format.replace("{locale}", &self.code)
    }
}

pub fn get_locale() -> Result<LocaleInfo, ModuleError> {
    let mut locale: LocaleInfo = LocaleInfo::new();

    locale.code = match env::var("LANG") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Locale", format!("Could not parse $LANG env variable: {}", e)))
    };

    Ok(locale)
}
