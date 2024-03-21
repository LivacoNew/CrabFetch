use std::io::ErrorKind::NotFound;
use core::str;
use std::{fmt::Display, process::Command};

use crate::Module;

pub struct GPUInfo {
    vendor: String,
    model: String,
    vram_mb: u32,
}
impl Module for GPUInfo {
    fn new() -> GPUInfo {
        GPUInfo {
            vendor: "".to_string(),
            model: "".to_string(),
            vram_mb: 0
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{vendor}", &self.vendor)
            .replace("{model}", &self.model)
            .replace("{vram_mb}", &self.vram_mb.to_string())
            .replace("{vram_gb}", &(self.vram_mb / 1024).to_string())
    }
}
impl Display for GPUInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} w/ {} mb", self.vendor, self.model, self.vram_mb)
    }
}

pub fn get_gpu() -> GPUInfo {
    let mut gpu = GPUInfo::new();

    // Grabs the info from glxinfo
    let output: Vec<u8> = match Command::new("glxinfo")
        .args(["-B"])
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    print!("GPU requires the 'glxinfo' command, which is not present!");
                } else {
                    print!("Unknown error while fetching GPU: {}", e);
                }

                return gpu
            },
        };

    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            print!("Unknown error while fetching GPU: {}", e);
            return gpu
        },
    };

    // Vendor is got from whatever line starts with "OpenGL vendor string"
    // Model is from line starting with "OpenGL renderer string", this will automatically search
    // for the vendor and remove it from the output
    // VRam is from line starting "Dedicated video memory"

    for line in contents.split("\n").collect::<Vec<&str>>() {
        let line = line.trim();
        if line.starts_with("OpenGL vendor string:") {
            // E.g OpenGL vendor string: AMD
            gpu.vendor = line[22..line.len()].to_string()
        }
        if line.starts_with("OpenGL renderer string:") {
            // E.g OpenGL renderer string: AMD Radeon RX 7800 XT (radeonsi, navi32, LLVM 17.0.6, DRM 3.57, 6.8.1-arch1-1)
            // Looks for the first ( and takes from there back
            gpu.model = line[24..line.find("(").unwrap()].replace(&gpu.vendor, "").trim().to_string()
        }
        if line.starts_with("Dedicated video memory:") {
            // E.g Dedicated video memory: 16384 MB
            gpu.vram_mb = match line[24..line.len() - 3].parse::<u32>() {
                Ok(r) => r,
                Err(e) => {
                    print!("Unable to parse GPU memory: {}", e);
                    0
                },
            };
        }
    }

    gpu
}
