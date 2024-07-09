use std::env::{self, VarError};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

pub struct EditorInfo {
    name: String,
    path: String,
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
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.editor.title_color).is_some() {
            title_color = &config.editor.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.editor.title_bold.is_some() {
            title_bold = config.editor.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.editor.title_italic.is_some() {
            title_italic = config.editor.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.editor.seperator.is_some() {
            seperator = config.editor.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.editor.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.editor.title_color).is_some() {
            title_color = config.editor.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.editor.title_bold.is_some() {
            title_bold = config.editor.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.editor.title_italic.is_some() {
            title_italic = config.editor.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.editor.seperator.is_some() {
            seperator = config.editor.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.editor.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.editor.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
    }
}

pub fn get_editor(fancy: bool) -> Result<EditorInfo, ModuleError> {
    let mut editor: EditorInfo = EditorInfo::new();

    editor.path = match env::var("EDITOR") {
        Ok(r) => {
            editor.name = r.split("/").last().unwrap().to_string();
            r
        },
        Err(e) => {
            if e == VarError::NotPresent {
                match env::var("VISUAL") {
                    Ok(r) => {
                        editor.name = r.split("/").last().unwrap().to_string();
                        r
                    },
                    Err(e) => {
                        if e == VarError::NotPresent {
                            "None".to_string()
                        } else {
                            return Err(ModuleError::new("Editor", format!("Could not parse $VISUAL env variable: {}", e)));
                        }
                    }
                }
            } else {
                return Err(ModuleError::new("Editor", format!("Could not parse $EDITOR env variable: {}", e)));
            }
        }
    };

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
