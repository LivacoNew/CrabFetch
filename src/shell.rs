use core::str;
use std::{fmt::Display, env};

use crate::Module;

pub struct ShellInfo {
    shell: String,
}
impl Module for ShellInfo {
    fn new() -> ShellInfo {
        ShellInfo {
            shell: "".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{shell}", &self.shell.split("/").collect::<Vec<&str>>().last().unwrap())
        .replace("{path}", &self.shell)
    }
}
impl Display for ShellInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.shell)
    }
}

pub fn get_shell() -> ShellInfo {
    let mut shell = ShellInfo::new();
    get_basic_info(&mut shell);

    shell
}

fn get_basic_info(shell: &mut ShellInfo) {
    // Just grabs whatevers in $SHELL
    shell.shell = match env::var("SHELL") {
        Ok(r) => r,
        Err(e) => {
            println!("WARNING: Could not parse $SHELL env variable: {}", e);
            "".to_string()
        }
    };
}
