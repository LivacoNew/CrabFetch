use core::str;
use std::fs;

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, package_managers::ManagerInfo, versions, ModuleError};

pub struct InitSystemInfo {
    name: String,
    path: String,
    version: String
}
#[derive(Deserialize)]
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

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.initsys.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.initsys.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.initsys.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.initsys.separator.as_ref().unwrap_or(&config.separator);

        Self::default_style(config, max_title_size, &config.initsys.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        config.initsys.format.replace("{name}", &self.name)
            .replace("{path}", &self.path)
            .replace("{version}", &self.version)
    }

    fn gen_info_flags(&self, config: &Configuration) -> u32 {
        todo!()
    }
}

pub fn get_init_system(fetch_version: bool, use_checksums: bool, package_managers: &ManagerInfo) -> Result<InitSystemInfo, ModuleError> {
    let mut initsys: InitSystemInfo = InitSystemInfo::new();

    // Just gets the symlink from /sbin/init 
    initsys.path = match fs::canonicalize("/sbin/init") {
        Ok(r) => r.display().to_string(),
        Err(e) => return Err(ModuleError::new("InitSys", format!("Failed to canonicalize /sbin/init symlink: {}", e)))
    };
    initsys.name = initsys.path.split('/')
            .last()
            .unwrap()
            .to_string();

    if fetch_version {
        initsys.version = versions::find_version(&initsys.path, Some(&initsys.name), use_checksums, package_managers).unwrap_or("Unknown".to_string());
    }

    Ok(initsys)
}
