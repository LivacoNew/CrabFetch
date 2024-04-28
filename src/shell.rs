use std::{env, fs, os::unix::process};

use serde::Deserialize;

use crate::{config_manager::{self, Configuration, CrabFetchColor, ModuleConfiguration, TOMLParseError}, Module, ModuleError};

pub struct ShellInfo {
    shell_name: String,
    shell_path: String,
}
#[derive(Deserialize)]
pub struct ShellConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub show_default_shell: bool
}
impl Default for ShellConfiguration {
    fn default() -> Self {
        ShellConfiguration {
            title: "Shell".to_string(),
            title_color: None,
            title_bold: None,
            title_italic: None,
            seperator: None,
            format: "{shell}".to_string(),
            show_default_shell: false
        }
    }
}
impl ModuleConfiguration for ShellConfiguration {
    fn apply_toml_line(&mut self, key: &str, value: &str) -> Result<(), crate::config_manager::TOMLParseError> {
        match key {
            "title" => self.title = config_manager::toml_parse_string(value)?,
            "title_color" => self.title_color = Some(config_manager::toml_parse_string_to_color(value)?),
            "title_bold" => self.title_bold = Some(config_manager::toml_parse_bool(value)?),
            "title_italic" => self.title_italic = Some(config_manager::toml_parse_bool(value)?),
            "seperator" => self.seperator = Some(config_manager::toml_parse_string(value)?),
            "format" => self.format = config_manager::toml_parse_string(value)?,
            "show_default_shell" => self.show_default_shell = config_manager::toml_parse_bool(value)?,
            _ => return Err(TOMLParseError::new("Unknown key.".to_string(), Some("Shell".to_string()), Some(key.to_string()), value.to_string()))
        }
        Ok(())
    }
}


impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell_name: "".to_string(),
            shell_path: "".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_length: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.shell.title_color).is_some() {
            title_color = &config.shell.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.shell.title_bold.is_some() {
            title_bold = config.shell.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.shell.title_italic.is_some() {
            title_italic = config.shell.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.shell.seperator.is_some() {
            seperator = config.shell.seperator.as_ref().unwrap();
        }

        self.default_style(config, max_title_length, &config.shell.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.shell.format.replace("{shell}", &self.shell_name)
            .replace("{path}", &self.shell_path)
    }
}

pub fn get_shell(show_default_shell: bool) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    if show_default_shell {
        return get_default_shell();
    }

    // Grabs the parent process and uses that
    // Goes into the /exe file and checks the symlink path, and uses that
    // Credit goes to FastFetch for this detection method - another tidbit of linux knowledge I was
    // unaware of
    let parent_pid: u32 = process::parent_id();
    let path: String = format!("/proc/{}/exe", parent_pid);
    let shell_path: String = match fs::canonicalize(&path) {
        Ok(r) => r.display().to_string(),
        Err(e) => return Err(ModuleError::new("Shell", format!("Failed to canonicalize {} symlink: {}", path, e)))
    };

    shell.shell_path = shell_path;
    shell.shell_name = shell.shell_path.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    Ok(shell)
}

fn get_default_shell() -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    shell.shell_path = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Shell", format!("Could not parse $SHELL env variable: {}", e)))
    };
    shell.shell_name = shell.shell_path.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    Ok(shell)
}
