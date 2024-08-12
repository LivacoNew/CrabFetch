use std::{mem, path::Path, time::Duration};

use humantime::format_duration;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util, ModuleError};

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

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.uptime.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.uptime.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.uptime.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.uptime.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.uptime.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.uptime.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.uptime.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let format: String = config.uptime.format.clone().unwrap_or("{time}".to_string());
        format.replace("{time}", &format_duration(self.uptime).to_string())
    }

    fn gen_info_flags(&self, config: &Configuration) -> u32 {
        todo!()
    }
}

pub fn get_uptime(sysinfo: &mut Option<libc::sysinfo>) -> Result<UptimeInfo, ModuleError> {
    let mut uptime: UptimeInfo = UptimeInfo::new();

    // Grabs from /proc/uptime
    let contents = match util::file_read(Path::new("/proc/uptime")) {
        Ok(r) => r,
        Err(_) => {
            // Backup to the sysinfo call
            use_syscall(sysinfo, &mut uptime);
            return Ok(uptime);
        },
    };
    uptime.uptime = match contents.split(' ').collect::<Vec<&str>>()[0].parse::<f64>() {
        Ok(r) => Duration::new(r.floor() as u64, 0),
        Err(e) => return Err(ModuleError::new("Uptime", format!("Could not parse /proc/uptime: {}", e))),
    };

    Ok(uptime)
}

fn use_syscall(sysinfo: &mut Option<libc::sysinfo>, uptime: &mut UptimeInfo) {
    let sysinfo_unwrap: libc::sysinfo = sysinfo.unwrap_or_else(|| {
        unsafe {
            let mut infobuf: libc::sysinfo = mem::zeroed();
            libc::sysinfo(&mut infobuf);
            *sysinfo = Some(infobuf);
            infobuf
        }
    });
    uptime.uptime = Duration::new(sysinfo_unwrap.uptime as u64, 0);
}
