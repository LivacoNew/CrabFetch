use std::{env, fs::File, io::{BufRead, BufReader}, path::Path};

use crate::module::ModuleError;

#[derive(Clone, Debug)]
pub struct GTKThemeCache {
    pub gtk2: String,
    pub gtk3: String,
    pub gtk4: String
}
impl Default for GTKThemeCache {
    fn default() -> Self {
        Self {
            gtk2: "Adwaita".to_string(),
            gtk3: "Adwaita".to_string(),
            gtk4: "Adwaita".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GTKIconCache {
    pub gtk2: String,
    pub gtk3: String,
    pub gtk4: String
}
impl Default for GTKIconCache {
    fn default() -> Self {
        Self {
            gtk2: "Adwaita".to_string(),
            gtk3: "Adwaita".to_string(),
            gtk4: "Adwaita".to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct GTKSettingsCache {
    themes: Option<GTKThemeCache>,
    icons: Option<GTKIconCache>
}
impl GTKSettingsCache {
    pub fn get_themes(&mut self) -> Result<GTKThemeCache, ModuleError> {
        if let Some(themes) = &self.themes {
            return Ok(themes.clone());
        }

        let config_path_str: String = if let Ok(r) = env::var("XDG_CONFIG_HOME") { r } else {
            // Let's try the home directory
            let mut home_dir: String = match env::var("HOME") {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Theme", format!("Unable to find suitable config folder; {e}")))
            };
            home_dir.push_str("/.config/");
            home_dir
        };
        let config_path: &Path = Path::new(&config_path_str);

        let mut themes: GTKThemeCache = GTKThemeCache::default();
        if let Some(gtk2) = read_gtk_property("gtk-theme-name", &config_path.join("gtk-2.0/settings.ini")) {
            themes.gtk2 = gtk2;
        }
        if let Some(gtk3) = read_gtk_property("gtk-theme-name", &config_path.join("gtk-3.0/settings.ini")) {
            themes.gtk3 = gtk3;
        }
        if let Some(gtk4) = read_gtk_property("gtk-theme-name", &config_path.join("gtk-4.0/settings.ini")) {
            themes.gtk4 = gtk4;
        }

        self.themes = Some(themes.clone());

        Ok(themes)
    }
    pub fn get_icons(&mut self) -> Result<GTKIconCache, ModuleError> {
        if let Some(icons) = &self.icons {
            return Ok(icons.clone());
        }

        let config_path_str: String = if let Ok(r) = env::var("XDG_CONFIG_HOME") { r } else {
            // Let's try the home directory
            let mut home_dir: String = match env::var("HOME") {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Theme", format!("Unable to find suitable config folder; {e}")))
            };
            home_dir.push_str("/.config/");
            home_dir
        };
        let config_path: &Path = Path::new(&config_path_str);

        let mut icons: GTKIconCache = GTKIconCache::default();
        if let Some(gtk2) = read_gtk_property("gtk-icon-theme-name", &config_path.join("gtk-2.0/settings.ini")) {
            icons.gtk2 = gtk2;
        }
        if let Some(gtk3) = read_gtk_property("gtk-icon-theme-name", &config_path.join("gtk-3.0/settings.ini")) {
            icons.gtk3 = gtk3;
        }
        if let Some(gtk4) = read_gtk_property("gtk-icon-theme-name", &config_path.join("gtk-4.0/settings.ini")) {
            icons.gtk4 = gtk4;
        }

        self.icons = Some(icons.clone());

        Ok(icons)
    }
}

fn read_gtk_property(property: &str, path: &Path) -> Option<String> {
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

        if !line.starts_with(property) {
            continue;
        }

        return Some(line[property.len() + 1..].to_string());
    }

    None
}
