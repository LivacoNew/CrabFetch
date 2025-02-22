use serde::Deserialize;

use crate::{common_sources::gtk::GTKSettingsCache, config_manager::Configuration, formatter::CrabFetchColor, module::Module, ModuleError};

pub struct IconThemeInfo {
    gtk2: String,
    gtk3: String,
    gtk4: String
}
#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct IconThemeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: Option<String>,
}
impl Module for IconThemeInfo {
    fn new() -> IconThemeInfo {
        IconThemeInfo {
            gtk2: "Adwaita".to_string(),
            gtk3: "Adwaita".to_string(),
            gtk4: "Adwaita".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.icontheme.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.icontheme.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.icontheme.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.icontheme.separator.as_ref().unwrap_or(&config.separator);

        let format: String = config.icontheme.format.clone().unwrap_or("{time}".to_string());
        let title: String = self.replace_placeholders(&config.icontheme.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) { 
        let title_color: &CrabFetchColor = config.icontheme.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.icontheme.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.icontheme.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.icontheme.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, &config.icontheme.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{gtk2}", &self.gtk2)
            .replace("{gtk3}", &self.gtk3)
            .replace("{gtk4}", &self.gtk4)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on icon theme module. This should never happen, please make a bug report!")
    }
}

pub fn get_icon_theme(gtk_settings: &mut GTKSettingsCache) -> Result<IconThemeInfo, ModuleError> {
    let mut icon_theme: IconThemeInfo = IconThemeInfo::new();

    if let Ok(icon_themes) = gtk_settings.get_icons() {
        icon_theme.gtk2 = icon_themes.gtk2_icon_theme;
        icon_theme.gtk3 = icon_themes.gtk3_icon_theme;
        icon_theme.gtk4 = icon_themes.gtk4_icon_theme;
    } else {
        return Err(ModuleError::new("Icon Themes", "Failed to read GTK settings.".to_string()));
    }

    Ok(icon_theme)
}
