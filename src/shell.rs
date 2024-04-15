use std::{env, fs::File, io::Read, os::unix::process};

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
    pub show_default_shell: bool
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
        CONFIG.shell.format.replace("{shell}", &self.shell_name)
    }
}

pub fn get_shell() -> ShellInfo {
    let mut shell: ShellInfo = ShellInfo::new();

    if CONFIG.shell.show_default_shell {
        return get_default_shell();
    }

    // Grabs the parent process and uses that
    let parent_pid: u32 = process::parent_id();
    let path: String = format!("/proc/{}/cmdline", parent_pid);

    let mut parent_stat: File = match File::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            log_error("Shell", format!("Can't open from {} - {}", path, e));
            return shell
        },
    };
    let mut contents: String = String::new();
    match parent_stat.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Shell", format!("Can't open from {} - {}", path, e));
            return shell
        },
    }

    shell.shell_name = contents.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    shell
}

pub fn get_default_shell() -> ShellInfo {
    let mut shell: ShellInfo = ShellInfo::new();

    // This is mostly here for terminal detection, but there's a config option to use this instead
    // too :)
    // This definitely isn't the old $SHELL grabbing code, no sir.
    shell.shell_name = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => {
            log_error("Shell", format!("Could not parse $SHELL env variable: {}", e));
            "".to_string()
        }
    }.split("/").collect::<Vec<&str>>().last().unwrap().to_string();

    shell
}
