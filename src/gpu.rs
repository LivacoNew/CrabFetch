use core::str;
use std::{fmt::Display, fs::File, path::Path, io::{ErrorKind::NotFound, Read, Write, Error}, process::Command, fs};

use crate::{log_error, Module};

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

pub fn get_gpu(ignore_cache: bool) -> GPUInfo {
    // Unlike other modules, GPU is cached!
    // This is because glxinfo takes ages to run, and users aren't going to be hot swapping GPUs
    // It caches into /tmp/crabfetch-gpu
    let mut gpu: GPUInfo = GPUInfo::new();
    if !ignore_cache {
        let cache_path: &Path = Path::new("/tmp/crabfetch-gpu");
        if cache_path.exists() {
            let cache_file: Result<File, Error> = File::open("/tmp/crabfetch-gpu");
            match cache_file {
                Ok(mut r) => {
                    let mut contents: String = String::new();
                    match r.read_to_string(&mut contents) {
                        Ok(_) => {
                            let split: Vec<&str> = contents.split("\n").collect();
                            gpu.vendor = split[0].to_string();
                            gpu.model = split[1].to_string();
                            gpu.vram_mb = split[2].parse::<u32>().unwrap();
                            return gpu;
                        },
                        Err(e) => {
                            log_error("GPU", format!("GPU Cache exists, but cannot read from it - {}", e));
                        },
                    }
                },
                Err(_) => {
                    // Assume the file is somehow corrupt
                    // Spooky but this time the spook won't delete your home directory (in theory) :)
                    drop(cache_file);
                    fs::remove_file("/tmp/crabfetch-gpu").expect("Unable to delete potentially corrupt GPU cache file.");
                }
            };
        }
    }

    // Grabs the info from glxinfo
    let output: Vec<u8> = match Command::new("glxinfo")
        .args(["-B"])
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    log_error("GPU", format!("GPU requires the 'glxinfo' command, which is not present!"));
                } else {
                    log_error("GPU", format!("Unknown error while fetching GPU: {}", e));
                }

                return gpu
            },
        };

    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("GPU", format!("Unknown error while fetching GPU: {}", e));
            return gpu
        },
    };

    // Vendor is got from whatever line starts with "OpenGL vendor string"
    // Model is from line starting with "OpenGL renderer string", this will automatically search
    // for the vendor and remove it from the output
    // VRam is from line starting "Dedicated video memory"

    for line in contents.split("\n").collect::<Vec<&str>>() {
        let line: &str = line.trim();
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
                    log_error("GPU", format!("Unable to parse GPU memory: {}", e));
                    0
                },
            };
        }
    }

    // Cache
    let mut file: File = match File::create("/tmp/crabfetch-gpu") {
        Ok(r) => r,
        Err(e) => {
            log_error("GPU", format!("Unable to cache GPU info: {}", e));
            return gpu;
        }
    };
    let write: String = format!("{}\n{}\n{}", gpu.vendor, gpu.model, gpu.vram_mb);
    match file.write(write.as_bytes()) {
        Ok(_) => {},
        Err(e) => {
            log_error("GPU", format!("Error writing to GPU cache: {}", e));
        }
    }

    gpu
}
