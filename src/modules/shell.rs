use std::env;

#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, common_sources::package_managers::ManagerInfo, proccess_info::ProcessInfo, util::is_flag_set_u32, versions, ModuleError};

pub struct ShellInfo {
    name: String,
    path: String,
    version: String,
}
#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
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

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.shell.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.shell.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.shell.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.shell.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.shell.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.shell.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.shell.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.shell.title
            .replace("{name}", "Unknown")
            .replace("{path}", "Unknown")
            .replace("{version}", "Unknown")
            .replace("{percentage}", "Unknown");

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
            info_flags |= SHELL_INFOFLAG_NAME;
            info_flags |= SHELL_INFOFLAG_PATH; // deps on path
        }
        if format.contains("{path}") {
            info_flags |= SHELL_INFOFLAG_PATH;
        }
        if format.contains("{version}") {
            // deps on all 3
            info_flags |= SHELL_INFOFLAG_NAME;
            info_flags |= SHELL_INFOFLAG_PATH;
            info_flags |= SHELL_INFOFLAG_VERSION;
        }

        info_flags
    }
}

const SHELL_INFOFLAG_NAME: u32 = 1;
const SHELL_INFOFLAG_PATH: u32 = 2;
const SHELL_INFOFLAG_VERSION: u32 = 4;

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
    "nu",
    "ion",
    "murex",
    "nushell",
    "oh",
    "powershell",
    "9base",
    "xonsh"
];

pub fn get_shell(config: &Configuration, package_managers: &ManagerInfo) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();
    let info_flags: u32 = ShellInfo::gen_info_flags(&config.shell.format);

    if config.shell.show_default_shell {
        return get_default_shell(info_flags, package_managers);
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
                Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find process cmdline: {}", e)))
            };
            if is_flag_set_u32(info_flags, SHELL_INFOFLAG_PATH) {
                shell.path = cmdline[1].to_string();
            }
            if is_flag_set_u32(info_flags, SHELL_INFOFLAG_NAME) {
                shell.name = shell.path.split('/').last().unwrap().to_string();
            }
        }
        #[cfg(not(feature = "android"))]
        {
            shell.name = match parent_process.get_process_name() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find process name: {e}")))
            };
        }

        if !KNOWN_SHELLS.contains(&shell.name.to_lowercase().as_str()) {
            parent_process = match parent_process.get_parent_process() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Shell", format!("Unable to get parent process: {e}")))
            };
            continue;
        }

        found = true;
    }


    // Android already has the path/name, regular people don't tho
    #[cfg(not(feature = "android"))]
    {
        if is_flag_set_u32(info_flags, SHELL_INFOFLAG_PATH) {
            shell.path = match parent_process.get_exe(true) {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Shell", format!("Failed to find exe path: {e}")))
            };
        }
    }

    if is_flag_set_u32(info_flags, SHELL_INFOFLAG_VERSION) {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), package_managers).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}

fn get_default_shell(info_flags: u32, package_managers: &ManagerInfo) -> Result<ShellInfo, ModuleError> {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    if is_flag_set_u32(info_flags, SHELL_INFOFLAG_PATH) {
        shell.path = match env::var("SHELL") {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Shell", format!("Could not parse $SHELL env variable: {e}")))
        };
        shell.path = match which::which(&shell.path) {
            Ok(r) => r.display().to_string(),
            Err(e) => return Err(ModuleError::new("Shell", format!("Could not find 'which' for {}: {e}", shell.path)))
        };
    }
    if is_flag_set_u32(info_flags, SHELL_INFOFLAG_NAME) {
        // apparently this is more efficient than just calling .to_string()
        // look idk man clippys yelling at me even though its messier
        shell.name = String::from(*shell.path.split('/')
            .collect::<Vec<&str>>()
            .last()
            .unwrap());
    }

    if is_flag_set_u32(info_flags, SHELL_INFOFLAG_VERSION) {
        shell.version = versions::find_version(&shell.path, Some(&shell.name), package_managers).unwrap_or("Unknown".to_string());
    }

    Ok(shell)
}
