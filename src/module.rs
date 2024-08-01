use std::{cmp::min, fmt::{Debug, Display}};

use colored::{ColoredString, Colorize};

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}};

pub trait Module {
    fn new() -> Self;
    fn style(&self, config: &Configuration, max_title_length: u64) -> String;
    fn unknown_output(config: &Configuration, max_title_length: u64) -> String;
    fn replace_placeholders(&self, config: &Configuration) -> String;

    // TODO: Move these params into some kinda struct or some shit idk, cus it just sucks
    fn default_style(config: &Configuration, max_title_len: u64, title: &str, title_color: &CrabFetchColor, title_bold: bool, title_italic: bool, separator: &str, value: &str) -> String {
        let mut str: String = String::new();

        // Title
        if !title.trim().is_empty() {
            let mut title: ColoredString = title_color.color_string(title);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            str.push_str(&title.to_string());
            // Inline value stuff
            if config.inline_values {
                for _ in 0..(max_title_len - min(title.chars().count() as u64, max_title_len)) {
                    str.push(' ');
                }
            }
            str.push_str(separator);
        }

        str.push_str(value);

        str
    }
    fn replace_color_placeholders(&self, str: &str) -> String {
        formatter::replace_color_placeholders(str)
    }
}

// A generic module error
pub struct ModuleError {
    module_name: String,
    message: String
}
impl ModuleError {
    pub fn new(module: &str, message: String) -> ModuleError {
        ModuleError {
            module_name: module.to_string(),
            message
        }
    }
}
impl Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module {} failed: {}", self.module_name, self.message)
    }
}
impl Debug for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module {} failed: {}", self.module_name, self.message)
    }
}
