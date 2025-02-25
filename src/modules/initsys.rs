use core::str;
use std::fs;

#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, common_sources::package_managers::ManagerInfo, proccess_info::ProcessInfo, util::is_flag_set_u32, versions, ModuleError};

pub struct InitSystemInfo {
    name: String,
    path: String,
    version: String
}
#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
pub struct InitSystemConfiguration {
    pub title: String,
    pub format: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
}
impl Module for InitSystemInfo {
    fn new() -> InitSystemInfo {
        InitSystemInfo {
            name: "Unknown".to_string(),
            path: "Unknown".to_string(),
            version: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.initsys.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.initsys.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.initsys.title
            .replace("{name}", "Unknown")
            .replace("{path}", "Unknown")
            .replace("{version}", "Unknown");

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
            info_flags |= INITSYS_INFOFLAG_NAME;
            info_flags |= INITSYS_INFOFLAG_PATH; // deps on path
        }
        if format.contains("{path}") {
            info_flags |= INITSYS_INFOFLAG_PATH;
        }
        if format.contains("{version}") {
            // deps on all 3
            info_flags |= INITSYS_INFOFLAG_NAME;
            info_flags |= INITSYS_INFOFLAG_PATH;
            info_flags |= INITSYS_INFOFLAG_VERSION;
        }

        info_flags
    }
}

const INITSYS_INFOFLAG_NAME: u32 = 1;
const INITSYS_INFOFLAG_PATH: u32 = 2;
const INITSYS_INFOFLAG_VERSION: u32 = 4;

pub fn get_init_system(config: &Configuration, package_managers: &ManagerInfo) -> Result<InitSystemInfo, ModuleError> {
    let mut initsys: InitSystemInfo = InitSystemInfo::new();
    let info_flags: u32 = InitSystemInfo::gen_info_flags(&config.initsys.format);

    // Reads the /cmdline of process 1, either using that or redirecting to it's symlink 
    // Thanks to https://superuser.com/a/1183819
    let mut process: ProcessInfo = ProcessInfo::new(1);

    if is_flag_set_u32(info_flags, INITSYS_INFOFLAG_PATH) {
        let path: String = match process.get_cmdline() {
            Ok(r) => r[0].to_string(),
            Err(e) => return Err(ModuleError::new("InitSys", format!("Failed to read from root process cmdline: {e}"))),
        };
        initsys.path = match fs::canonicalize(&path) {
            Ok(r) => r.display().to_string(),
            Err(e) => return Err(ModuleError::new("InitSys", format!("Failed to canonicalize {path} symlink: {e}")))
        };
    }
    if is_flag_set_u32(info_flags, INITSYS_INFOFLAG_NAME) {
        initsys.name = initsys.path.split('/')
            .last()
            .unwrap()
            .to_string();
    }

    if is_flag_set_u32(info_flags, INITSYS_INFOFLAG_VERSION) {
        if initsys.name == "init" {
            // Likely sysvinit, which can't be version detected
            initsys.version = "Unknown".to_string();
        } else {
            initsys.version = versions::find_version(&initsys.path, Some(&initsys.name), package_managers).unwrap_or("Unknown".to_string());
        }
    }

    Ok(initsys)
}
