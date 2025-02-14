use serde::Deserialize;

use crate::{ascii_art, config_manager::{self, Configuration}, formatter::CrabFetchColor};

#[derive(Deserialize)]
pub struct AsciiConfiguration {
    pub display: bool,
    pub side: String,
    pub colors: Vec<CrabFetchColor>,
    pub margin: u16,
}
#[derive(Debug)]
pub enum AsciiMode {
    Raw,
    Solid,
    Band,
    BandVertical,
}

// Return type is the ascii & the maximum length of it
pub fn find_ascii(os: &str) -> (String, u16) {
    // Will first confirm if theres a ascii override file
    let user_override: Option<String> = config_manager::check_for_ascii_override();
    if user_override.is_some() {
        let mut length: u16 = 0;
        user_override.as_ref().unwrap().split('\n').for_each(|x| {
            let len: usize = x.chars().count();
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
    if line.chars().count() < *target_length as usize {
        line.push_str(&" ".repeat(*target_length as usize - line.chars().count()));
    }

    line
}
