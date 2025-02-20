use serde::Deserialize;

use crate::{common_sources::gtk::GTKSettingsCache, config_manager::Configuration, formatter::CrabFetchColor, module::Module, ModuleError};

pub struct ThemeInfo {
    gtk2: String,
    gtk3: String,
    gtk4: String
}
#[derive(Deserialize)]
pub struct ThemeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: Option<String>,
}
impl Module for ThemeInfo {
    fn new() -> ThemeInfo {
        ThemeInfo {
            gtk2: "Adwaita".to_string(),
            gtk3: "Adwaita".to_string(),
            gtk4: "Adwaita".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.theme.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.theme.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.theme.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.theme.separator.as_ref().unwrap_or(&config.separator);

        let format: String = config.theme.format.clone().unwrap_or("{time}".to_string());
        let title: String = self.replace_placeholders(&config.theme.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) { 
        let title_color: &CrabFetchColor = config.theme.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.theme.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.theme.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.theme.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, &config.theme.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{gtk2}", &self.gtk2)
            .replace("{gtk3}", &self.gtk3)
            .replace("{gtk4}", &self.gtk4)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on theme module. This should never happen, please make a bug report!")
    }
}

pub fn get_theme(gtk_settings: &mut GTKSettingsCache) -> Result<ThemeInfo, ModuleError> {
    let mut theme: ThemeInfo = ThemeInfo::new();

    if let Ok(themes) = gtk_settings.get_themes() {
        theme.gtk2 = themes.gtk2_theme;
        theme.gtk3 = themes.gtk3_theme;
        theme.gtk4 = themes.gtk4_theme;
    } else {
        return Err(ModuleError::new("Themes", "Failed to read GTK settings.".to_string()));
    }

    Ok(theme)
}
