use core::str;
use std::{env, process::Command};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, syscalls::SyscallCache, util::is_flag_set_u32, ModuleError};

pub struct HostnameInfo {
    username: String,
    hostname: String,
}
#[derive(Deserialize)]
pub struct HostnameConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String
}
impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "Unknown".to_string(),
            hostname: "Unknown".to_string(),
        }
    }
    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.hostname.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.hostname.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.hostname.format.replace("{username}", &self.username)
            .replace("{hostname}", &self.hostname)
            .to_string()
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        // model and vendor are co-dependent
        if format.contains("{hostname}") {
            info_flags |= HOSTNAME_INFOFLAG_HOSTNAME;
        }
        if format.contains("{username}") {
            info_flags |= HOSTNAME_INFOFLAG_USERNAME;
        }

        info_flags
    }
}

const HOSTNAME_INFOFLAG_HOSTNAME: u32 = 1;
const HOSTNAME_INFOFLAG_USERNAME: u32 = 2;

pub fn get_hostname(config: &Configuration, syscall_cache: &mut SyscallCache) -> Result<HostnameInfo, ModuleError> {
    let mut hostname: HostnameInfo = HostnameInfo::new();
    let info_flags: u32 = HostnameInfo::gen_info_flags(&config.hostname.format);

    // We'll try the safe way first, then the backup way
    // This is purely cus reading that env variable is faster
    if is_flag_set_u32(info_flags, HOSTNAME_INFOFLAG_USERNAME) {
        hostname.username = match env::var("USER") {
            Ok(r) => r,
            Err(_) => syscall_cache.get_passwd_cached().name
        };
    }


    // Hostname
    // Unlike username, reading the hostname data as a syscall is faster than the file
    if is_flag_set_u32(info_flags, HOSTNAME_INFOFLAG_HOSTNAME) {
        hostname.hostname = syscall_cache.get_uname_cached().nodename;
    }

    Ok(hostname)
}

fn _backup_to_hostname_command(hostname: &mut HostnameInfo) -> Result<(), ModuleError> {
    // If all else is fucked, it'll come here
    let output: Vec<u8> = match Command::new("hostname")
        .output() {
            Ok(r) => r.stdout,
            Err(_) => return Err(ModuleError::new("Hostname", "Can't find hostname source.".to_string())),
        };

    hostname.hostname = match String::from_utf8(output) {
        Ok(r) => r.trim().to_string(),
        Err(_) => return Err(ModuleError::new("Hostname", "Can't find hostname source.".to_string())),
    };

    Ok(())
}
