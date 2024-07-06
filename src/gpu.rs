use core::str;
use std::{fs::{self, File, ReadDir}, io::{BufRead, BufReader, Error, ErrorKind::NotFound, Read, Write}, path::Path, process::Command};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, Module, ModuleError};

pub struct GPUInfo {
    vendor: String,
    model: String,
    vram_mb: u32,
}
#[derive(Deserialize, Clone)]
pub enum GPUMethod {
    GLXInfo,
    PCISysFile
}
impl ToString for GPUMethod {
    fn to_string(&self) -> String {
        match &self {
            GPUMethod::GLXInfo => "glxinfo".to_string(),
            GPUMethod::PCISysFile => "pcisysfile".to_string()
       }
    }
}
#[derive(Deserialize)]
pub struct GPUConfiguration {
    pub method: GPUMethod,
    pub cache: bool,

    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub use_ibis: Option<bool>,
    pub format: String
}

impl Module for GPUInfo {
    fn new() -> GPUInfo {
        GPUInfo {
            vendor: "".to_string(),
            model: "".to_string(),
            vram_mb: 0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.gpu.title_color).is_some() {
            title_color = config.gpu.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.gpu.title_bold.is_some() {
            title_bold = config.gpu.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.gpu.title_italic.is_some() {
            title_italic = config.gpu.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.gpu.seperator.is_some() {
            seperator = config.gpu.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.gpu.title, title_color, title_bold, title_italic, &seperator, &value)
    }

    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.gpu.title_color).is_some() {
            title_color = config.gpu.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.gpu.title_bold.is_some() {
            title_bold = config.gpu.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.gpu.title_italic.is_some() {
            title_italic = config.gpu.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.gpu.seperator.is_some() {
            seperator = config.gpu.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.gpu.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut use_ibis: bool = config.use_ibis;
        if config.gpu.use_ibis.is_some() {
            use_ibis = config.gpu.use_ibis.unwrap();
        }
        config.gpu.format.replace("{vendor}", &self.vendor)
            .replace("{model}", &self.model)
            .replace("{vram}", &formatter::auto_format_bytes((self.vram_mb * 1000) as u64, use_ibis, 0))
    }
}

pub fn get_gpus(method: GPUMethod, use_cache: bool) -> Result<Vec<GPUInfo>, ModuleError> {
    // Unlike other modules, GPU is cached!
    // This is because glxinfo takes ages to run, and users aren't going to be hot swapping GPUs
    // It caches into /tmp/crabfetch-gpu
    let mut gpus: Vec<GPUInfo> = Vec::new();

    // Get the GPU info via the selected method
    if use_cache {
        let cache_path: &Path = Path::new("/tmp/crabfetch-gpu");
        if cache_path.exists() {
            let cache_file: Result<File, Error> = File::open("/tmp/crabfetch-gpu");
            let file: Option<File> = match cache_file {
                Ok(r) => Some(r),
                Err(_) => None, // Assume the file is somehow corrupt
            };

            if file.is_some() {
                let mut contents: String = String::new();
                match file.as_ref().unwrap().read_to_string(&mut contents) {
                    Ok(_) => {
                        let gpu_entry: Vec<&str> = contents.split("\n\n").collect();
                        for entry in gpu_entry {
                            if entry.is_empty() {
                                continue;
                            }
                            let mut gpu: GPUInfo = GPUInfo::new();
                            let split: Vec<&str> = entry.split("\n").collect();
                            if split[0] == method.to_string() {
                                gpu.vendor = split[1].to_string();
                                gpu.model = split[2].to_string();
                                gpu.vram_mb = split[3].parse::<u32>().unwrap();
                            }
                            gpus.push(gpu);
                        }
                        return Ok(gpus);
                    },
                    Err(e) => return Err(ModuleError::new("GPU", format!("GPU Cache exists, but cannot read from it - {}", e))),
                }
            }
            // We've not returned yet so we can only assume it's fucked
            // Spooky but this time the spook won't delete your home directory (in theory) :)
            drop(file);
            fs::remove_file("/tmp/crabfetch-gpu").expect("Unable to delete potentially corrupt GPU cache file.");
        }
    }


    let filled: Result<(), ModuleError> = match method {
        GPUMethod::PCISysFile => fill_from_pcisysfile(&mut gpus),
        GPUMethod::GLXInfo => {
            let mut gpu: GPUInfo = GPUInfo::new();
            fill_from_glxinfo(&mut gpu)
        },
    };
    match filled {
        Ok(_) => {},
        Err(e) => return Err(e)
    }

    // Cache
    if use_cache {
        let mut file: File = match File::create("/tmp/crabfetch-gpu") {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Unable to cache GPU info: {}", e)))
        };
        let mut write: String = String::new();
        for gpu in &gpus {
            write.push_str(&format!("{}\n{}\n{}\n{}\n\n", method.to_string(), gpu.vendor, gpu.model, gpu.vram_mb));
        }
        match file.write(write.as_bytes()) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("GPU", format!("Error writing to GPU cache: {}", e)))
        }
    }

    Ok(gpus)
}

fn fill_from_pcisysfile(gpus: &mut Vec<GPUInfo>) -> Result<(), ModuleError> {
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
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from /sys/bus/pci/devices: {}", e))),
    };
    for dev_dir in dir {
        // This does the following;
        // Checks "class" for a HEX value that begins with 0x03
        // (https://github.com/torvalds/linux/blob/master/include/linux/pci_ids.h#L38)
        // It then parses from "vendor" "device" and "mem_info_vram_total" to get all the info it
        // needs
        let d = match dev_dir {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Failed to open directory: {}", e))),
        };
        // println!("{}", d.path().to_str().unwrap());

        let mut class_file: File = match File::open(d.path().join("class")) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Failed to open file {}: {}", d.path().join("class").to_str().unwrap(), e))),
        };
        let mut contents: String = String::new();
        match class_file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
        }

        if !contents.starts_with("0x03") {
            // Not a display device
            // And yes, I'm doing this check with a string instead of parsing it w/ a AND fuck you.
            continue
        }

        // Vendor/Device
        let mut vendor_file: File = match File::open(d.path().join("vendor")) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Failed to open file {}: {}", d.path().join("vendor").to_str().unwrap(), e))),
        };
        let mut vendor_str: String = String::new();
        match vendor_file.read_to_string(&mut vendor_str) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
        }
        let vendor: &str = vendor_str[2..].trim();

        let mut device_file: File = match File::open(d.path().join("device")) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Failed to open file {}: {}", d.path().join("device").to_str().unwrap(), e))),
        };
        let mut device_str: String = String::new();
        match device_file.read_to_string(&mut device_str) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
        }
        let device: &str = device_str[2..].trim();
        let dev_data: (String, String) = match search_pci_ids(vendor, device) {
            Ok(r) => r,
            Err(e) => return Err(e)
        };

        let mut gpu: GPUInfo = GPUInfo::new();
        gpu.vendor = dev_data.0;
        if vendor == "1002" { // AMD
            gpu.model = search_amd_model(device)?;
        } else {
            gpu.model = dev_data.1;
        }

        // Finally, Vram
        match File::open(d.path().join("mem_info_vram_total")) {
            Ok(mut r) => {
                let mut vram_str: String = String::new();
                match r.read_to_string(&mut vram_str) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e)));
                    },
                }
                gpu.vram_mb = (vram_str.trim().parse::<u64>().unwrap() / 1024 / 1024) as u32;
            },
            Err(_) => {}, // dw about it, this can happen on VM's for some reason
        };

        gpus.push(gpu);
    }

    return Ok(());
}
fn search_pci_ids(vendor: &str, device: &str) -> Result<(String, String), ModuleError> {
    // Search all known locations
    // /usr/share/hwdata/pci.ids
    let mut ids_path: Option<&str> = None;
    if Path::new("/usr/share/hwdata/pci.ids").exists() {
        ids_path = Some("/usr/share/hwdata/pci.ids");
    } else if Path::new("/usr/share/misc/pci.ids").exists() {
        ids_path = Some("/usr/share/misc/pci.ids");
    }

    if ids_path.is_none() {
        return Err(ModuleError::new("GPU", format!("Could not find an appropriate path for getting PCI ID info.")));
    }

    let file: File = match File::open(ids_path.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("GPU", format!("Can't read from {} - {}", ids_path.unwrap(), e)));
        },
    };
    let buffer: BufReader<File> = BufReader::new(file);

    // parsing this file is weird
    let mut vendor_result: String = String::new();
    let mut device_result: String = String::new();
    // Find the vendor ID + device in the list
    let vendor_term: String = String::from(vendor);
    let dev_term: String = String::from('\t') + device;
    let mut in_vendor: bool = false;
    for line in buffer.lines() { // NOTE: Looping here alone takes 1.7ms - This needs to be reduced
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.trim().starts_with("#") {
            continue
        }

        if in_vendor && line.chars().next().is_some() {
            in_vendor = line.chars().next().unwrap().is_whitespace();
            if !in_vendor {
                // Assume we missed it
                break
            }
        }

        if line.starts_with(&vendor_term) && vendor_result.is_empty() {
            // Assume the first hit of this is our full vendor name
            vendor_result = line[vendor_term.len()..].trim().to_string();
            in_vendor = true;
        } else if line.starts_with(&dev_term) && in_vendor {
            // And here's the device name
            device_result = line[dev_term.len()..].trim().to_string();
            break
        }
    }

    if device_result.is_empty() {
        device_result += device;
    }

    Ok((vendor_result.to_string(), device_result.to_string()))
}
// TODO: Revision ID searching too
fn search_amd_model(device: &str) -> Result<String, ModuleError> {
    let mut ids_path: Option<&str> = None;
    if Path::new("/usr/share/libdrm/amdgpu.ids").exists() {
        ids_path = Some("/usr/share/libdrm/amdgpu.ids");
    }
    if ids_path.is_none() {
        return Err(ModuleError::new("GPU", format!("Could not find an appropriate path for getting AMD PCI ID info.")));
    }

    let file: File = match File::open(ids_path.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            return Err(ModuleError::new("GPU", format!("Can't read from {} - {}", ids_path.unwrap(), e)));
        },
    };
    let buffer: BufReader<File> = BufReader::new(file);

    let mut device_result: String = String::new();
    let dev_term: String = device.to_lowercase().to_string();
    for line in buffer.lines() { 
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.trim().starts_with("#") {
            continue
        }

        if line.to_lowercase().starts_with(&dev_term) {
            device_result = line.split("\t").nth(2).unwrap().trim().to_string();
            break
        }
    }

    if device_result.is_empty() {
        device_result += device;
    }

    Ok(device_result.to_string())
}


fn fill_from_glxinfo(gpu: &mut GPUInfo) -> Result<(), ModuleError> {
    let output: Vec<u8> = match Command::new("glxinfo")
        .args(["-B"])
        .output() {
            Ok(r) => r.stdout,
            Err(e) => {
                if NotFound == e.kind() {
                    return Err(ModuleError::new("GPU", format!("GPU requires the 'glxinfo' command, which is not present!")));
                } else {
                    return Err(ModuleError::new("GPU", format!("Unknown error while fetching GPU: {}", e)));
                }
            },
        };

    let contents: String = match String::from_utf8(output) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("GPU", format!("Unknown error while fetching GPU: {}", e))),
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
                Err(e) => return Err(ModuleError::new("GPU", format!("Unable to parse GPU memory: {}", e))),
            };
        }
    }

    Ok(())
}
