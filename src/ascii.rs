use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor};

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
pub fn get_ascii(os: &str) -> (String, u16) {
    (String::new(), 0)
}

pub fn get_ascii_line(current_line: usize, ascii_split: &[&str], target_length: &u16, config: &Configuration) -> String {
    String::new()
}
