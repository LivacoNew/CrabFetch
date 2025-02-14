use std::{cmp::min, collections::HashMap};

use colored::ColoredString;
use serde::Deserialize;

use crate::{ascii_art, config_manager::{self, Configuration}, formatter::CrabFetchColor};

#[derive(Deserialize)]
pub struct AsciiConfiguration {
    pub display: bool,
    pub side: String,
    pub margin: u16,
    pub mode: AsciiMode,
    // Coloring options 
    // Done this way because I kid you not I could not find a way to make an multi-type thing for
    // the Config crate
    pub solid_color: CrabFetchColor,
    pub band_colors: Vec<CrabFetchColor>
}
#[derive(Debug, Deserialize, PartialEq)]
pub enum AsciiMode {
    Raw,
    OS,
    Solid,
    Band
}

// Return type is the ascii & the maximum length of it
pub fn find_ascii(os: &str) -> (String, u16) {
    // Will first confirm if theres a ascii override file
    let user_override: Option<String> = config_manager::check_for_ascii_override();
    if user_override.is_some() {
        let mut length: u16 = 0;
        user_override.as_ref().unwrap().split('\n').for_each(|x| {
            let stripped = strip_ansi_escapes::strip_str(x);
            let len: usize = stripped.chars().count();
            if len > length as usize { length = len as u16 }
        });
        return (user_override.unwrap(), length)
    }
    let os: &str = &os.replace('"', "").to_lowercase();

    let ascii: (&str, u16) = match os {
        "arch" => ascii_art::ARCH,
        "debian" => ascii_art::DEBIAN,
        "ubuntu" => ascii_art::UBUNTU,
        "fedora" => ascii_art::FEDORA,
        "void" => ascii_art::VOID,
        "endeavouros" => ascii_art::ENDEAVOUR,
        "linuxmint" => ascii_art::MINT,
        "elementary" => ascii_art::ELEMENTARY,
        "zorin" => ascii_art::ZORIN,
        "manjaro" => ascii_art::MANJARO,
        "pop" => ascii_art::POPOS,
        "opensuse-tumbleweed" => ascii_art::OPENSUSE,
        "opensuse-leap" => ascii_art::OPENSUSE,
        "bazzite" => ascii_art::BAZZITE,
        "rocky" => ascii_art::ROCKYLINUX,
        "kali" => ascii_art::KALI,
        "almalinux" => ascii_art::ALMA,
        "android" => ascii_art::ANDROID,
        "garuda" => ascii_art::GARUDA,
        _ => ("", 0)
    };

    // I blame rust not letting me make const strings
    let ascii_string: String = ascii.0.to_string();
    (ascii_string, ascii.1)
}

pub fn get_ascii_line(current_line: usize, ascii_split: &[&str], target_length: &u16, config: &Configuration) -> String {
    let mut line: String = String::new();

    if ascii_split.len() > current_line {
        line.push_str(ascii_split[current_line]);
    }

    let ansi_stripped = strip_ansi_escapes::strip_str(&line);
    if ansi_stripped.chars().count() < *target_length as usize {
        line.push_str(&" ".repeat(*target_length as usize - ansi_stripped.chars().count()));
    }

    if config.ascii.mode != AsciiMode::Raw {
        line = match config.ascii.mode {
            AsciiMode::Raw => line,
            AsciiMode::OS => color_solid(&line, config), // main func sets the solid color to be the os color in this case
            AsciiMode::Solid => color_solid(&line, config),
            AsciiMode::Band => color_band_vertical(&line, current_line, ascii_split.len(), config)
        }
    }

    line
}

fn color_solid(line: &str, config: &Configuration) -> String {
    config.ascii.solid_color.color_string(line).to_string()
}

fn color_band_vertical(line: &str, current_line: usize, ascii_length: usize, config: &Configuration) -> String {
    let percentage: f32 = current_line as f32 / (ascii_length - 1) as f32;
    let total_colors: usize = config.ascii.band_colors.len();
    let index: usize = min((((total_colors - 1) as f32) * percentage).round() as usize, total_colors - 1);

    let colored: ColoredString = config.ascii.band_colors.get(index as usize).unwrap().color_string(line);
    colored.to_string()
}
