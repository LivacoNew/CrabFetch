use std::env;

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.locale.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.locale.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.locale.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.locale.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.locale.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.locale.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.locale.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.locale.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.locale.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.locale.title, title_color, title_bold, title_italic, seperator, "Unknown")
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
