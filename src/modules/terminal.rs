use std::{env, fs, os::unix::process};

#[cfg(feature = "android")]
use std::path::Path;

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, proccess_info::ProcessInfo, versions, Module, ModuleError};

pub struct TerminalInfo {
    name: String,
    path: String,
    version: String
}
#[derive(Deserialize)]
pub struct TerminalConfiguration {
    pub title: String,
    pub format: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub chase_ssh_pts: bool
}
impl Module for TerminalInfo {
    fn new() -> TerminalInfo {
        TerminalInfo {
            name: "Unknown".to_string(),
            path: "Unknown".to_string(),
            version: "Unknown".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.terminal.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.terminal.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.terminal.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.terminal.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.terminal.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.terminal.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.terminal.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.terminal.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.terminal.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.terminal.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.terminal.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }
}

pub fn get_terminal(chase_ssh_tty: bool, fetch_version: bool) -> Result<TerminalInfo, ModuleError> {
    let mut terminal: TerminalInfo = TerminalInfo::new();

    #[cfg(feature = "android")]
    if env::consts::OS == "android" && Path::new("/data/data/com.termux/files/").exists() { // TODO: Does this still work in other emulators?
        terminal.name = "Termux".to_string();
        return Ok(terminal);
    }

    // This is just a rust-ified & slightly more robust solution from https://askubuntu.com/a/508047
    // Find the terminal's PID by going up through every shell level
    let mut terminal_pid: Option<u32> = None;

    let mut loops = 0; // always use protection against infinite loops kids
    let mut parent_pid: u32 = process::parent_id();
    let mut shell_level: u8 = match env::var("SHLVL") {
        Ok(r) => match r.parse::<u8>() {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Terminal", format!("Could not parse $SHLVL env variable: {}", e)))
        },
        Err(e) => return Err(ModuleError::new("Terminal", format!("Could not get $SHLVL env variable: {}", e)))
    };
    while shell_level > 0 {
        if loops > 10 {
            return Err(ModuleError::new("Terminal", "Terminal PID loop ran for more than 10 iterations! Either I'm in a infinite loop, or you're >10 shells deep, in which case you're a moron.".to_string()));
        }
        loops += 1;

        let mut process: ProcessInfo = ProcessInfo::new(parent_pid);

        if shell_level == 1 {
            terminal_pid = Some(match process.get_parent_pid() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Terminal", format!("Can't parse terminal pid: {}", e))),
            });
        } else {
            // go up a level
            parent_pid = match process.get_parent_pid() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Terminal", format!("Can't parse parent pid: {}", e))),
            };
        }

        shell_level -= 1;
    }


    if terminal_pid.is_none() {
        return Err(ModuleError::new("Terminal", format!("Was unsuccessfull in finding Terminal's PID, last checked; {}", parent_pid)));
    }
    let terminal_pid: u32 = terminal_pid.unwrap();
    let mut terminal_process: ProcessInfo = ProcessInfo::new(terminal_pid);
    if !terminal_process.is_valid() {
        return Err(ModuleError::new("Terminal", "Unable to find terminal process".to_string()));
    }
    terminal.name = match terminal_process.get_process_name() {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Terminal", format!("Can't get process name: {}", e))),
    };

    // Fix for gnome terminal coming out as gnome-terminal-server
    if terminal.name.trim() == "gnome-terminal-server" {
        terminal.name = "GNOME Terminal".to_string();
    }
    // Fix for elementaryos terminal being shitty
    if terminal.name.trim() == "io.elementary.terminal" {
        terminal.name = "Elementary Terminal".to_string();
    }
 
    if terminal.name.trim().replace('\0', "") == "sshd:" {
        if !chase_ssh_tty {
            terminal.name = "SSH Terminal".to_string();
        } else {
            // Find the tty number in the current /stat and just construct it from that
            // Taken from the comment on https://unix.stackexchange.com/a/77797 by "user723" ...
            // get a more creative username man
            terminal.name = match fs::canonicalize("/proc/self/fd/0") {
                Ok(r) => r.display().to_string(),
                Err(e) => return Err(ModuleError::new("Terminal", format!("Failed to canonicalize /proc/self/fd/0 symlink: {}", e)))
            };
        }
    }

    terminal.path = match terminal_process.get_exe(true) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Terminal", format!("Can't get process exe: {}", e))),
    };

    if fetch_version {
        terminal.version = versions::find_version(&terminal.path, Some(&terminal.name)).unwrap_or("Unknown".to_string());
    }

    Ok(terminal)
}
