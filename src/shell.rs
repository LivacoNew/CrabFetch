use std::{fmt::Display, env};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, Module, CONFIG};

pub struct ShellInfo {
    shell_name: String,
}
#[derive(Deserialize)]
pub struct ShellConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell_name: "".to_string(),
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.shell.title_color).is_some() {
            title_color = &CONFIG.shell.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.shell.title_bold).is_some() {
            title_bold = CONFIG.shell.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.shell.title_italic).is_some() {
            title_italic = CONFIG.shell.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.shell.seperator.is_some() {
            seperator = CONFIG.shell.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.shell.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        CONFIG.shell.format.replace("{shell}", &self.shell_name.split("/").collect::<Vec<&str>>().last().unwrap())
            .replace("{path}", &self.shell_name)
    }
}
impl Display for ShellInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.shell_name)
    }
}

pub fn get_shell() -> ShellInfo {
    let mut shell: ShellInfo = ShellInfo::new();

    // Just grabs whatevers in $SHELL
    shell.shell_name = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => {
            log_error("Shell", format!("Could not parse $SHELL env variable: {}", e));
            "".to_string()
        }
    };

    shell
}
