use core::str;
use std::{fmt::Display, fs::{self, File, ReadDir}, io::{Error, ErrorKind::NotFound, Read, Write}, path::Path, process::Command};

use crate::{config_manager::GPUMethod, log_error, Module, ARGS, CONFIG};

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
    // Unlike other modules, GPU is cached!
    // This is because glxinfo takes ages to run, and users aren't going to be hot swapping GPUs
    // It caches into /tmp/crabfetch-gpu
    let mut gpu: GPUInfo = GPUInfo::new();
    if !ARGS.ignore_cache && CONFIG.gpu_cache {
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

    // Get the GPU info via the selected method
    match CONFIG.gpu_method {
        GPUMethod::PCISysFile => fill_from_pcisysfile(&mut gpu),
        GPUMethod::GLXInfo => fill_from_glxinfo(&mut gpu),
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

fn fill_from_pcisysfile(gpu: &mut GPUInfo) {
    // This scans /sys/bus/pci/devices/ and checks the class to find the first display adapter it
    // can
    // This needs expanded at a later date
    //
    // This uses pci.ids inside pciutils to identify devices:
    // https://man.archlinux.org/man/core/pciutils/pci.ids.5.en
    //
    // To prevent having to relicence to GPL I don't distribute a copy of this, and simply have to
    // rely on the host's copy. Problem then becomes that different distros seem to place this file
    // in different places
    // I'll try to find it in as many places as possible but ultimately can't cover every place. If
    // you know the places, make a PR/Issue and i'll add it in. Fucking hate licences that work
    // like this but oh well.

    let dir: ReadDir = match fs::read_dir("/sys/bus/pci/devices") {
        Ok(r) => r,
        Err(e) => {
            log_error("GPU", format!("Can't read from /sys/bus/pci/devices: {}", e));
            return
        },
    };
    for dev_dir in dir {
        // This does the following;
        // Checks "class" for a HEX value that begins with 0x03
        // (https://github.com/torvalds/linux/blob/master/include/linux/pci_ids.h#L38)
        // It then parses from "vendor" "device" and "mem_info_vram_total" to get all the info it
        // needs
        let d = match dev_dir {
            Ok(r) => r,
            Err(e) => {
                log_error("GPU", format!("Failed to open directory: {}", e));
                continue
            },
        };
        // println!("{}", d.path().to_str().unwrap());

        let mut class_file: File = match File::open(d.path().join("class")) {
            Ok(r) => r,
            Err(e) => {
                log_error("GPU", format!("Failed to open file {}: {}", d.path().join("class").to_str().unwrap(), e));
                continue;
            },
        };
        let mut contents: String = String::new();
        match class_file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => {
                log_error("GPU", format!("Can't read from file: {}", e));
                return
            },
        }

        if !contents.starts_with("0x03") {
            // Not a display device
            // And yes, I'm doing this check with a string instead of parsing it w/ a AND fuck you.
            continue
        }

        // Vendor/Device
        let mut vendor_file: File = match File::open(d.path().join("vendor")) {
            Ok(r) => r,
            Err(e) => {
                log_error("GPU", format!("Failed to open file {}: {}", d.path().join("vendor").to_str().unwrap(), e));
                continue;
            },
        };
        let mut vendor_str: String = String::new();
        match vendor_file.read_to_string(&mut vendor_str) {
            Ok(_) => {},
            Err(e) => {
                log_error("GPU", format!("Can't read from file: {}", e));
                return
            },
        }
        let vendor: &str = vendor_str[2..].trim();

        let mut device_file: File = match File::open(d.path().join("device")) {
            Ok(r) => r,
            Err(e) => {
                log_error("GPU", format!("Failed to open file {}: {}", d.path().join("device").to_str().unwrap(), e));
                continue;
            },
        };
        let mut device_str: String = String::new();
        match device_file.read_to_string(&mut device_str) {
            Ok(_) => {},
            Err(e) => {
                log_error("GPU", format!("Can't read from file: {}", e));
                return
            },
        }
        let device: &str = device_str[2..].trim();
        let dev_data: (String, String) = search_pci_ids(vendor, device);

        gpu.vendor = dev_data.0;
        gpu.model = dev_data.1;

        // Finally, Vram
        let mut vram_file: File = match File::open(d.path().join("mem_info_vram_total")) {
            Ok(r) => r,
            Err(e) => {
                log_error("GPU", format!("Failed to open file {}: {}", d.path().join("mem_info_vram_total").to_str().unwrap(), e));
                continue;
            },
        };
        let mut vram_str: String = String::new();
        match vram_file.read_to_string(&mut vram_str) {
            Ok(_) => {},
            Err(e) => {
                log_error("GPU", format!("Can't read from file: {}", e));
                return
            },
        }
        gpu.vram_mb = (vram_str.trim().parse::<u64>().unwrap() / 1024 / 1024) as u32;
        return
    }
}
fn search_pci_ids(vendor: &str, device: &str) -> (String, String) {
    // Search all known locations
    // /usr/share/hwdata/pci.ids
    let mut ids_path: Option<&str> = None;
    if Path::new("/usr/share/hwdata/pci.ids").exists() {
        ids_path = Some("/usr/share/hwdata/pci.ids");
    }

    if ids_path.is_none() {
        log_error("GPU", format!("Could not find an appropriate path for getting PCI ID info."));
        return ("".to_string(), "".to_string())
    }

    let mut file: File = match File::open(ids_path.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            log_error("GPU", format!("Can't read from {} - {}", ids_path.unwrap(), e));
            return ("".to_string(), "".to_string())
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("GPU", format!("Can't read from {} - {}", ids_path.unwrap(), e));
            return ("".to_string(), "".to_string())
        },
    }

    // parsing this file is weird
    let mut vendor_result: String = String::new();
    let mut device_result: String = String::new();
    // Find the vendor ID + device in the list
    let vendor_term: String = String::from(vendor);
    let dev_term: String = String::from(device);
    let mut in_vendor: bool = false;
    for line in contents.split("\n") {
        if line.trim().starts_with("#") {
            // Comment
            continue
        }
        if in_vendor && line.chars().next().is_some() {
            in_vendor = line.chars().next().unwrap().is_whitespace();
        }

        let line: &str = line.trim();

        if line.starts_with(&vendor_term) && vendor_result.is_empty() {
            // Assume the first hit of this is our full vendor name
            vendor_result = line[4..].trim().to_string();
            in_vendor = true;
        }
        if line.starts_with(&dev_term) && in_vendor {
            // And here's the device name
            device_result = line[4..].trim().to_string();
        }
    }

    (vendor_result.to_string(), device_result.to_string())
}

fn fill_from_glxinfo(gpu: &mut GPUInfo) {
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

                return
            },
        };

    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => {
            log_error("GPU", format!("Unknown error while fetching GPU: {}", e));
            return
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
}
