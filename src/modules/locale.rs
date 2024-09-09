use std::env;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, module::Module, ModuleError};

pub struct LocaleInfo {
    language: String,
    encoding: String,
}
#[derive(Deserialize, JsonSchema)]
pub struct LocaleConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String
}
impl Module for LocaleInfo {
    fn new() -> LocaleInfo {
        LocaleInfo {
            language: "Unknown".to_string(),
            encoding: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.locale.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.locale.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.locale.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.locale.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.locale.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.locale.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.locale.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.locale.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.locale.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.locale.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.locale.title
            .replace("{language}", "Unknown")
            .replace("{encoding}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{language}", &self.language)
            .replace("{encoding}", &self.encoding)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on locale module. This should never happen, please make a bug report!")
    }
}

pub fn get_locale() -> Result<LocaleInfo, ModuleError> {
    // no info flags here as it's all from the same source
    let mut locale: LocaleInfo = LocaleInfo::new();

    let raw: String = match env::var("LANG") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Locale", format!("Could not parse $LANG env variable: {e}")))
    };
    let raw_split: Vec<&str> = raw.split('.').collect();
    locale.language = String::from(*raw_split.first().unwrap_or(&locale.language.as_ref()));
    locale.encoding = String::from(*raw_split.get(1).unwrap_or(&locale.encoding.as_ref()));

    Ok(locale)
}
