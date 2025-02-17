use std::{env, fs::File, io::{BufRead, BufReader}, path::Path};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, ModuleError};

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

pub fn get_theme() -> Result<ThemeInfo, ModuleError> {
    let mut theme: ThemeInfo = ThemeInfo::new();

    let config_path_str: String = match env::var("XDG_CONFIG_HOME") {
        Ok(r) => r,
        Err(_) => {
            // Let's try the home directory
            let mut home_dir: String = match env::var("HOME") {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Theme", format!("Unable to find suitable config folder; {e}")))
            };
            home_dir.push_str("/.config/");
            home_dir
        }
    };
    let config_path: &Path = Path::new(&config_path_str);

    if let Some(gtk2) = read_gtk_settings(&config_path.join("gtk-2.0/settings.ini")) {
        theme.gtk2 = gtk2;
    }
    if let Some(gtk3) = read_gtk_settings(&config_path.join("gtk-3.0/settings.ini")) {
        theme.gtk3 = gtk3;
    }
    if let Some(gtk4) = read_gtk_settings(&config_path.join("gtk-4.0/settings.ini")) {
        theme.gtk4 = gtk4;
    }

    Ok(theme)
}

fn read_gtk_settings(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }

    let file: File = match File::open(path) {
        Ok(r) => r,
        Err(_) => return None
    };
    let buffer: BufReader<File> = BufReader::new(file);
    
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if !line.starts_with("gtk-theme-name") {
            continue;
        }

        return Some(line[15..].to_string());
    }

    None
}
