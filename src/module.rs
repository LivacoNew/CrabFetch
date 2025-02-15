use std::fmt::{Debug, Display};

use colored::{ColoredString, Colorize};

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}};

pub trait Module {
    fn new() -> Self;
    fn style(&self, config: &Configuration) -> (String, String);
    fn unknown_output(config: &Configuration) -> (String, String);
    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String;
    fn gen_info_flags(format: &str) -> u32;

    // TODO: Move these params into some kinda struct or some shit idk, cus it just sucks
    fn default_style(_: &Configuration, title: &str, title_color: &CrabFetchColor, title_bold: bool, title_italic: bool, separator: &str, value: &str) -> (String, String) {
        let mut title_final: String = String::new();
        let mut value_final: String = String::new();

        // Title
        if !title.trim().is_empty() {
            let mut title: ColoredString = title_color.color_string(title);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            title_final.push_str(&title.to_string());
            value_final.push_str(separator);
        }

        value_final.push_str(value);

        (title_final, value_final)
    }
    fn replace_color_placeholders(&self, str: &str, config: &Configuration) -> String {
        formatter::replace_color_placeholders(str, config)
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
