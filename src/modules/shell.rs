use std::env;

#[cfg(feature = "android")]
use std::{fs::File, io::Read};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, package_managers::ManagerInfo, proccess_info::ProcessInfo, versions, module::Module, ModuleError};

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
    pub separator: Option<String>,
    pub format: String,
    pub show_default_shell: bool
}
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            name: "Unknown".to_string(),
            path: "Unknown".to_string(),
            version: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.shell.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.shell.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.shell.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.shell.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.shell.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }
}

// A list of known shells, the idea being that we keep going up in parent processes until we
// encouter one
// This prevents stuff like sudo, scripts or running crabfetch as a child process in any way messing us up
// Compiled from info here https://wiki.archlinux.org/title/Command-line_shell
pub const KNOWN_SHELLS: &[&str] = &[
    "bash", 
    "dash",
    "ksh",
    "nsh",
    "oil",
    "yash",
    "zsh", 
    "tcsh",
    "closh",
    "elvish",
    "fish",
    "ion",
    "murex",
    "nushell",
    "oh",
    "powershell",
    "9base",
    "xonsh"
];

pub fn get_shell(show_default_shell: bool, fetch_version: bool, use_checksums: bool, package_managers: &ManagerInfo) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    if show_default_shell {
        return get_default_shell(fetch_version, use_checksums, package_managers);
    }

    // Goes up until we hit one of our known shells
    let mut parent_process: ProcessInfo = ProcessInfo::new_from_parent();
    let mut found: bool = false;
    let mut loop_limit: u8 = 0;
    while !found {
        if loop_limit >= 10 {
            return Err(ModuleError::new("Shell", "Shell parent process loop ran for more than 10 iterations! Either I'm in a infinite loop, or you're >10 subprocesses deep, in which case you're a moron.".to_string()));
        }
        loop_limit += 1;

        // "attributes on expressions are experimental" 
        // rust... the issue has been open since 2014... can you just fucking add it please because
        // the random scope braces I have to add here are genuinely horrendous
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
            shell.name = match parent_process.get_process_name() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find process name: {}", e)))
            };
        }

        if !KNOWN_SHELLS.contains(&shell.name.to_lowercase().as_str()) {
            parent_process = match parent_process.get_parent_process() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Shell", format!("Unable to get parent process: {}", e)))
            };
            continue;
        }

        found = true;
    }


    // Android already has the path/name, regular people don't tho
    #[cfg(not(feature = "android"))]
    {
        shell.path = match parent_process.get_exe(true) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find exe path: {}", e)))
        };
    }

    if fetch_version {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), use_checksums, package_managers).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}

fn get_default_shell(fetch_version: bool, use_checksums: bool, package_managers: &ManagerInfo) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    shell.path = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Shell", format!("Could not parse $SHELL env variable: {}", e)))
    };
    shell.path = match which::which(&shell.path) {
        Ok(r) => r.display().to_string(),
        Err(e) => return Err(ModuleError::new("Shell", format!("Could not find 'which' for {}: {}", shell.path, e)))
    };
    shell.name = shell.path.split('/')
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_string();

    if fetch_version {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), use_checksums, package_managers).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}
