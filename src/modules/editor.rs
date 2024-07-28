use std::env;

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, versions, Module, ModuleError};

pub struct EditorInfo {
    name: String,
    path: String,
    version: String
}
#[derive(Deserialize)]
pub struct EditorConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub fancy: bool
}
impl Module for EditorInfo {
    fn new() -> EditorInfo {
        EditorInfo {
            name: "".to_string(),
            path: "".to_string(),
            version: "".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.editor.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.editor.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.editor.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.editor.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.editor.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.editor.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.editor.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.editor.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.editor.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.editor.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.editor.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }
}

pub fn get_editor(fancy: bool, fetch_version: bool, use_checksums: bool) -> Result<EditorInfo, ModuleError> {
    let mut editor: EditorInfo = EditorInfo::new();

    let env_value: String = match env::var("EDITOR") {
        Ok(r) => r,
        Err(_) => {
            match env::var("VISUAL") {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Editor", format!("Could not parse $EDITOR or $VISUAL variable: {}", e)))
            }
        },
    };

    editor.path = match which::which(&env_value) {
        Ok(r) => r.display().to_string(),
        Err(e) => return Err(ModuleError::new("Editor", format!("Could not find 'which' for {}: {}", env_value, e)))
    };
    editor.name = editor.path.split('/').last().unwrap().to_string();
    if fetch_version {
        editor.version = versions::find_version(&editor.path, Some(&editor.name), use_checksums).unwrap_or("Unknown".to_string());
    }

    // Convert the name to a fancy variant
    // I don't like hardcoding like this, but otherwise the result looks dumb
    if fancy {
        editor.name = match editor.name.as_str() {
            "vi" => "VI".to_string(),
            "vim" => "Vim".to_string(),
            "nvim" => "NeoVim".to_string(),
            "nano" => "GNU Nano".to_string(),
            "emacs" => "Emacs".to_string(),
            "gedit" => "GEdit".to_string(),
            _ => editor.name
        };
    }


    Ok(editor)
}
