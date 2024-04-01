use core::str;
use std::{fmt::Display, env};

use crate::{log_error, Module};

pub struct ShellInfo {
    shell_name: String,
}
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell_name: "".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{shell}", &self.shell_name.split("/").collect::<Vec<&str>>().last().unwrap())
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
