use std::{path::Path, time::Duration};

use humantime::format_duration;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, syscalls::SyscallCache, util, ModuleError};

pub struct UptimeInfo {
    uptime: Duration,
}
#[derive(Deserialize)]
pub struct UptimeConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: Option<String>,
}
impl Module for UptimeInfo {
    fn new() -> UptimeInfo {
        UptimeInfo {
            uptime: Duration::new(0, 0),
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.uptime.separator.as_ref().unwrap_or(&config.separator);

        let format: String = config.uptime.format.clone().unwrap_or("{time}".to_string());
        let title: String = self.replace_placeholders(&config.uptime.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) { 
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.uptime.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.uptime.title.replace("{time}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{time}", &format_duration(self.uptime).to_string())
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on uptime module. This should never happen, please make a bug report!")
    }
}

pub fn get_uptime(syscall_cache: &mut SyscallCache) -> Result<UptimeInfo, ModuleError> {
    let mut uptime: UptimeInfo = UptimeInfo::new();

    // Grabs from /proc/uptime
    let contents = match util::file_read(Path::new("/proc/uptime")) {
        Ok(r) => r,
        Err(_) => {
            // Backup to the sysinfo call
            use_syscall(syscall_cache, &mut uptime);
            return Ok(uptime);
        },
    };
    uptime.uptime = match contents.split(' ').collect::<Vec<&str>>()[0].parse::<f64>() {
        Ok(r) => Duration::new(r.floor() as u64, 0),
        Err(e) => return Err(ModuleError::new("Uptime", format!("Could not parse /proc/uptime: {}", e))),
    };

    Ok(uptime)
}

fn use_syscall(syscall_cache: &mut SyscallCache, uptime: &mut UptimeInfo) {
    let sysinfo_unwrap: libc::sysinfo = syscall_cache.get_sysinfo_cached();
    uptime.uptime = Duration::new(sysinfo_unwrap.uptime as u64, 0);
}
