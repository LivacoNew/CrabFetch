use core::str;
use std::fmt::Display;

use crate::Module;

pub struct Segment {
    title: String
}
impl Module for Segment {
    fn new() -> Segment {
        Segment {
            title: "".to_string()
        }
    }

    fn style(&self) -> String {
        todo!()
    }

    fn replace_placeholders(&self) -> String {
        todo!()
    }
    // fn format(&self, _: &str, _: u32) -> String {
    //     todo!()
    //     // format.replace("{vendor}", &self.vendor)
    //     //     .replace("{model}", &self.model)
    //     //     .replace("{vram_mb}", &self.vram_mb.to_string())
    //     //     .replace("{vram_gb}", &(self.vram_mb / 1024).to_string())
    // }
}
impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "------{}", self.title)
    }
}

