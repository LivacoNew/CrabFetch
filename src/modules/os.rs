use core::str;
use std::path::Path;

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, syscalls::SyscallCache, util::{self, is_flag_set_u32}, ModuleError};

pub struct OSInfo {
    distro: String,
    pub distro_id: String,
    kernel: String,
}
#[derive(Deserialize)]
pub struct OSConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub newline_kernel: bool,
    pub kernel_title: String,
    pub kernel_format: String
}
impl Module for OSInfo {
    fn new() -> OSInfo {
        OSInfo {
            distro: "Unknown".to_string(),
            distro_id: "Unknown".to_string(),
            kernel: "Unknown".to_string(),
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.os.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.os.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.os.title
            .replace("{distro}", "Unknown")
            .replace("{kernel}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{distro}", &self.distro)
            .replace("{kernel}", &self.kernel)
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{distro}") {
           info_flags |= OS_INFOFLAG_DISTRO;
        }
        if format.contains("{kernel}") {
            info_flags |= OS_INFOFLAG_KERNEL;
        }

        info_flags
    }
}
impl OSInfo {
    // Identical to the regular style method, but placeholder's in the kernel instead
    pub fn style_kernel(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.os.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.os.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.os.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.os.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&config.os.kernel_format.replace("{kernel}", &self.kernel), config);

        Self::default_style(config, &config.os.kernel_title, title_color, title_bold, title_italic, separator, &value)
    }
}

const OS_INFOFLAG_DISTRO: u32 = 1;
const OS_INFOFLAG_KERNEL: u32 = 2;

pub fn get_os(config: &Configuration, syscall_cache: &mut SyscallCache) -> Result<OSInfo, ModuleError> {
    let mut os: OSInfo = OSInfo::new();

    let mut format: String = config.os.format.to_string();
    if config.os.newline_kernel {
        format.push_str(&config.os.kernel_format);
    }
    let info_flags: u32 = OSInfo::gen_info_flags(&format);

    // Grabs the distro name from /etc/os-release
    // Grabs the kernel release from /proc/sys/kernel/osrelease

    // Distro
    if is_flag_set_u32(info_flags, OS_INFOFLAG_DISTRO) || config.ascii.display {
        // Android 
        #[cfg(feature = "android")]
        if env::consts::OS == "android" {
            let props = AndroidSystemProperties::new();
            // https://github.com/termux/termux-api/issues/448#issuecomment-927345222
            if let Some(val) = props.get("ro.build.version.release_or_preview_display") {
                os.distro = format!("Android {}", val);
            } else {
                os.distro = "Android".to_string();
            }
            os.distro_id = "android".to_string();
        } else {
            parse_os_release(&mut os)?;
        }

        #[cfg(not(feature = "android"))]
        parse_os_release(&mut os)?;
    }

    // Kernel
    if is_flag_set_u32(info_flags, OS_INFOFLAG_KERNEL) {
        os.kernel = syscall_cache.get_uname_cached().release;
    }

    Ok(os)
}

fn parse_os_release(os: &mut OSInfo) -> Result<(), ModuleError> {
    let contents = match util::file_read(Path::new("/etc/os-release")) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("OS", format!("Can't read from /etc/os-release - {}", e))),
    };
    for line in contents.trim().split('\n').collect::<Vec<&str>>() {
        if line.starts_with("PRETTY_NAME=") {
            os.distro = line[13..line.len() - 1].to_string();
            continue;
        }
        if line.starts_with("ID=") {
            os.distro_id = line[3..line.len()].trim().to_string();
            continue;
        }
    }

    Ok(())
}
