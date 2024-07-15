use std::{env, fs, os::unix::process};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell_name: "".to_string(),
            shell_path: "".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.shell.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.shell.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.shell.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.shell.title, title_color, title_bold, title_italic, seperator, "Unknown")
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
    #[cfg(feature = "android")]
    let shell_path: String = if env::consts::OS == "android" {
        format!("/proc/{}/cmdline", parent_pid)
    } else {
        let path: String = format!("/proc/{}/exe", parent_pid);
        match fs::canonicalize(&path) {
            Ok(r) => r.display().to_string(),
            Err(e) => return Err(ModuleError::new("Shell", format!("Failed to canonicalize {} symlink: {}", path, e)))
        }
    };

    shell.shell_path = shell_path;
    shell.shell_name = shell.shell_path.split('/')
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_string();

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
    shell.shell_name = shell.shell_path.split('/')
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_string();

    Ok(shell)
}
