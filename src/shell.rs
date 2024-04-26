use std::{env, fs, os::unix::process};

use serde::Deserialize;

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

pub struct ShellInfo {
    shell_name: String,
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
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell_name: "".to_string(),
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

    shell.shell_name = shell_path.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    Ok(shell)
}

pub fn get_default_shell() -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    shell.shell_name = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Shell", format!("Could not parse $SHELL env variable: {}", e)))
    }.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    Ok(shell)
}
