use std::env;

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, package_managers::ManagerInfo, util::is_flag_set_u32, versions, ModuleError};

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
    pub separator: Option<String>,
    pub format: String,
    pub fancy: bool
}
impl Module for EditorInfo {
    fn new() -> EditorInfo {
        EditorInfo {
            name: "Unknown".to_string(),
            path: "Unknown".to_string(),
            version: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.editor.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.editor.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.editor.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.editor.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.editor.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.editor.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.editor.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.editor.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.editor.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.editor.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.editor.title
            .replace("{name}", "Unknown")
            .replace("{path}", "Unknown")
            .replace("{version}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{name}") {
            info_flags |= EDITOR_INFOFLAG_NAME;
            info_flags |= EDITOR_INFOFLAG_PATH // deps on path
        }
        if format.contains("{path}") {
            info_flags |= EDITOR_INFOFLAG_PATH
        }
        if format.contains("{version}") {
            // deps on all 3
            info_flags |= EDITOR_INFOFLAG_NAME;
            info_flags |= EDITOR_INFOFLAG_PATH;
            info_flags |= EDITOR_INFOFLAG_VERSION
        }

        info_flags
    }
}

const EDITOR_INFOFLAG_NAME: u32 = 1;
const EDITOR_INFOFLAG_PATH: u32 = 2;
const EDITOR_INFOFLAG_VERSION: u32 = 4;

pub fn get_editor(config: &Configuration, package_managers: &ManagerInfo) -> Result<EditorInfo, ModuleError> {
    let mut editor: EditorInfo = EditorInfo::new();
    let info_flags: u32 = EditorInfo::gen_info_flags(&config.editor.format);

    let env_value: String = match env::var("EDITOR") {
        Ok(r) => r,
        Err(_) => {
            match env::var("VISUAL") {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Editor", format!("Could not parse $EDITOR or $VISUAL variable: {e}")))
            }
        },
    };

    if is_flag_set_u32(info_flags, EDITOR_INFOFLAG_PATH) {
        editor.path = match which::which(&env_value) {
            Ok(r) => r.display().to_string(),
            Err(e) => return Err(ModuleError::new("Editor", format!("Could not find 'which' for {env_value}: {e}")))
        };
    }
    if is_flag_set_u32(info_flags, EDITOR_INFOFLAG_NAME) {
        editor.name = editor.path.split('/').last().unwrap().to_string();
    }
    if is_flag_set_u32(info_flags, EDITOR_INFOFLAG_VERSION) {
        editor.version = versions::find_version(&editor.path, Some(&editor.name), package_managers).unwrap_or("Unknown".to_string());
    }

    // Convert the name to a fancy variant
    // I don't like hardcoding like this, but otherwise the result looks dumb
    if config.editor.fancy {
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
