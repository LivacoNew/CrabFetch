use std::{env, os::unix::process};

#[cfg(feature = "android")]
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, proccess_info::ProcessInfo, Module, ModuleError, versions};

pub struct ShellInfo {
    name: String,
    path: String,
    version: String,
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
            name: "".to_string(),
            path: "".to_string(),
            version: "".to_string()
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
        config.shell.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }
}

pub fn get_shell(show_default_shell: bool, fetch_version: bool, use_checksums: bool) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    if show_default_shell {
        return get_default_shell(fetch_version, use_checksums);
    }

    // Just assumes the parent process
    let parent_pid: u32 = process::parent_id();
    let mut parent_process: ProcessInfo = ProcessInfo::new(parent_pid);

    #[cfg(feature = "android")]
    {
        let cmdline: Vec<String> = match parent_process.get_cmdline() {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("OS", format!("Can't read from {} cmdline - {}", parent_pid, e))),
        };
        shell.path = cmdline[1].to_string();
        shell.name = shell.path.split('/').last().unwrap().to_string();
    }
    #[cfg(not(feature = "android"))]
    {
        shell.path = match parent_process.get_exe(true) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find exe path: {}", e)))
        };
        shell.name = match parent_process.get_process_name() {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find process name: {}", e)))
        };
    }

    if fetch_version {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), use_checksums).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}

fn get_default_shell(fetch_version: bool, use_checksums: bool) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    shell.path = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Shell", format!("Could not parse $SHELL env variable: {}", e)))
    };
    shell.name = shell.path.split('/')
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_string();

    if fetch_version {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), use_checksums).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}
