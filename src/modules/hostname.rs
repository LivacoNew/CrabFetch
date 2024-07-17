use core::str;
use std::{env, ffi::CStr, fs::File, io::Read, mem, process::Command};

use libc::{geteuid, getpwuid, uname};
use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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
    pub seperator: Option<String>,
    pub format: String
}
impl Module for HostnameInfo {
    fn new() -> HostnameInfo {
        HostnameInfo {
            username: "".to_string(),
            hostname: "".to_string(),
        }
    }
    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.hostname.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.hostname.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.hostname.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.hostname.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.hostname.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.hostname.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.hostname.format.replace("{username}", &self.username)
            .replace("{hostname}", &self.hostname)
            .to_string()
    }
}

pub fn get_hostname() -> Result<HostnameInfo, ModuleError> {
    let mut hostname: HostnameInfo = HostnameInfo::new();

    // We'll try the safe way first, then the backup way
    // This is purely cus reading that env variable is faster
    hostname.username = match env::var("USER") {
        Ok(r) => r,
        Err(_) => {
            // syscall dangerous time
            match get_username_unsafe() {
                Ok(r) => r,
                Err(_) => return Err(ModuleError::new("Hostname", "WARNING: Could not get username (env variable nor syscall)".to_string()))
            }
        }
    };


    // Hostname
    // Unlike username, reading the hostname data as a syscall is faster than the file
    hostname.hostname = match get_hostname_unsafe() {
        Ok(r) => r,
        Err(_) => {
            let mut file: File = match File::open("/etc/hostname") {
                Ok(r) => r,
                Err(_) => {
                    match backup_to_hostname_command(&mut hostname) {
                        Ok(_) => return Ok(hostname),
                        Err(e) => return Err(e),
                    }
                },
            };
            let mut contents: String = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {},
                Err(_) => {
                    match backup_to_hostname_command(&mut hostname) {
                        Ok(_) => return Ok(hostname),
                        Err(e) => return Err(e),
                    }
                },
            }
            if contents.trim().is_empty() {
                match backup_to_hostname_command(&mut hostname) {
                    Ok(_) => return Ok(hostname),
                    Err(e) => return Err(e),
                }
            }
            contents.trim()
                .lines()
                .filter(|x| !x.starts_with('#'))
                .collect()
        },
    };

    Ok(hostname)
}

fn get_username_unsafe() -> Result<String, ()> { // No error type as I simply need to know if it failed or not to backup to $USER, the type of err doesn't matter really
    let name: String;

    unsafe { // i do find it funny that no-ones made guarenteed safe c lib bindings in a language designed entirely around memory safety lol
        let user_id: u32 = geteuid();
        let passwd: *mut libc::passwd = getpwuid(user_id);
        if passwd.is_null() {
            return Err(()) // null pointer
        }

        name = match CStr::from_ptr((*passwd).pw_name).to_str() {
            Ok(r) => r.to_string(),
            Err(_) => return Err(()),
        };
    }

    Ok(name)
}
fn get_hostname_unsafe() -> Result<String, ()> {
    let hostname: String;

    unsafe {
        let mut name_buff: libc::utsname = mem::zeroed();
        uname(&mut name_buff);
        hostname = CStr::from_ptr(name_buff.nodename.as_ptr()).to_str().unwrap().to_string();
    }

    Ok(hostname)
}
fn backup_to_hostname_command(hostname: &mut HostnameInfo) -> Result<(), ModuleError> {
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