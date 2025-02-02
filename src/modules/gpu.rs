use core::str;
use std::{fs::{self, File, ReadDir}, io::{BufRead, BufReader}, path::Path};

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, util::{self, is_flag_set_u32}, ModuleError};

#[derive(Clone)]
pub struct GPUInfo {
    index: Option<u8>,
    vendor: String,
    model: String,
    vram_mb: u32,
}
#[derive(Deserialize)]
pub struct GPUConfiguration {
    pub amd_accuracy: bool,
    pub ignore_disabled_gpus: bool,

    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub use_ibis: Option<bool>,
    pub format: String
}

impl Module for GPUInfo {
    fn new() -> GPUInfo {
        GPUInfo {
            index: None,
            vendor: "Unknown".to_string(),
            model: "Unknown".to_string(),
            vram_mb: 0
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.gpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.gpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.gpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.gpu.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.gpu.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.gpu.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }

    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.gpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.gpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.gpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.gpu.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.gpu.title
            .replace("{vendor}", "Unknown")
            .replace("{model}", "Unknown")
            .replace("{vram}", "Unknown")
            .replace("{index}", "0").to_string();

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let use_ibis: bool = config.gpu.use_ibis.unwrap_or(config.use_ibis);

        text.replace("{vendor}", &self.vendor)
            .replace("{model}", &self.model)
            .replace("{vram}", &formatter::auto_format_bytes((self.vram_mb * 1000) as u64, use_ibis, 0))
            .replace("{index}", &self.index.unwrap_or(0).to_string())
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        // model and vendor are co-dependent
        if format.contains("{vendor}") {
            info_flags |= GPU_INFOFLAG_VENDOR;
            info_flags |= GPU_INFOFLAG_MODEL;
        }
        if format.contains("{model}") {
            info_flags |= GPU_INFOFLAG_MODEL;
            info_flags |= GPU_INFOFLAG_VENDOR;
        }
        if format.contains("{vram}") {
            info_flags |= GPU_INFOFLAG_VRAM
        }

        info_flags
    }
}
impl GPUInfo {
    pub fn set_index(&mut self, index: u8) {
        self.index = Some(index);
    }
}

const GPU_INFOFLAG_VENDOR: u32 = 1;
const GPU_INFOFLAG_MODEL: u32 = 2;
const GPU_INFOFLAG_VRAM: u32 = 4;

pub fn get_gpus(config: &Configuration) -> Result<Vec<GPUInfo>, ModuleError> {
    let mut gpus: Vec<GPUInfo> = Vec::new();
    let info_flags: u32 = GPUInfo::gen_info_flags(&config.gpu.format);

    match fill_from_pcisysfile(&mut gpus, config.gpu.amd_accuracy, config.gpu.ignore_disabled_gpus, info_flags) {
        Ok(_) => {},
        Err(e) => return Err(e)
    }

    Ok(gpus)
}

fn fill_from_pcisysfile(gpus: &mut Vec<GPUInfo>, amd_accuracy: bool, ignore_disabled: bool, info_flags: u32) -> Result<(), ModuleError> {
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


        match util::file_read(&d.path().join("class")) {
            Ok(r) => {
                if !r.starts_with("0x03") {
                    // Not a display device
                    // And yes, I'm doing this check with a string instead of parsing it w/ a AND fuck you.
                    continue
                }
            },
            Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
        };

        if ignore_disabled {
            match util::file_read(&d.path().join("enable")) {
                Ok(r) => {
                    if r.trim() == "0" {
                        continue;
                    }
                },
                Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
            };
        }

        let mut gpu: GPUInfo = GPUInfo::new();
        // Vendor/Device
        if is_flag_set_u32(info_flags, GPU_INFOFLAG_MODEL) || is_flag_set_u32(info_flags, GPU_INFOFLAG_VENDOR) {
            let vendor_id: String = match util::file_read(&d.path().join("vendor")) {
                Ok(r) => r[2..].trim().to_string(),
                Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
            };
            let device_id: String = match util::file_read(&d.path().join("device")) {
                Ok(r) => r[2..].trim().to_string(),
                Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
            };
            let device_data: (String, String) = match search_pci_ids(&vendor_id, &device_id) {
                Ok(r) => r,
                Err(e) => return Err(e)
            };

            // TODO: Just directly search AMD, not the first pci.ids file
            gpu.vendor = device_data.0;
            if vendor_id == "1002" && amd_accuracy { // AMD
                let revision_id: String = match util::file_read(&d.path().join("revision")) {
                    Ok(r) => r[2..].trim().to_string(),
                    Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {}", e))),
                };
                gpu.model = match search_amd_model(&device_id, &revision_id)? {
                    Some(r) => r,
                    None => device_data.1,
                };
            } else {
                gpu.model = device_data.1;
            }
        }

        // Finally, Vram
        if is_flag_set_u32(info_flags, GPU_INFOFLAG_VRAM) {
            if let Ok(r) = util::file_read(&d.path().join("mem_info_vram_total")) {
                gpu.vram_mb = (r.trim().parse::<u64>().unwrap() / 1024 / 1024) as u32;
            }
        }

        gpus.push(gpu);
    }

    Ok(())
}
fn search_pci_ids(vendor: &str, device: &str) -> Result<(String, String), ModuleError> {
    // Search all known locations
    let ids_path: &Path = match util::find_first_path_exists(vec![
        Path::new("/usr/share/hwdata/pci.ids"),
        Path::new("/usr/share/misc/pci.ids")
    ]) {
        Some(r) => r,
        None => return Err(ModuleError::new("GPU", "Could not find an appropriate path for getting PCI ID info.".to_string()))
    };

    let file: File = match File::open(ids_path) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from {} - {}", ids_path.display(), e))),
    };
    let buffer: BufReader<File> = BufReader::new(file);

    // parsing this file is weird
    let mut vendor_result: String = String::new();
    let mut device_result: String = String::new();
    // Find the vendor ID + device in the list
    let vendor_term: String = String::from(vendor);
    let dev_term: String = String::from('\t') + device;
    let mut in_vendor: bool = false;
    for line in buffer.lines() { 
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.trim().starts_with('#') {
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
fn search_amd_model(device: &str, revision: &str) -> Result<Option<String>, ModuleError> {
    let ids_path: &Path = match util::find_first_path_exists(vec![
        Path::new("/usr/share/libdrm/amdgpu.ids")
    ]) {
        Some(r) => r,
        None => return Err(ModuleError::new("GPU", "Could not find an appropriate path for getting AMD PCI ID info.".to_string()))
    };

    let file: File = match File::open(ids_path) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from {} - {}", ids_path.display(), e))),
    };
    let buffer: BufReader<File> = BufReader::new(file);

    let mut device_result: String = String::new();
    let dev_term: String = format!("{},\t{},\t", device, revision).to_lowercase().to_string();
    for line in buffer.lines() { 
        if line.is_err() {
            continue;
        }
        let line: String = line.unwrap();

        if line.trim().starts_with('#') {
            continue
        }

        if line.to_lowercase().starts_with(&dev_term) {
            device_result = line[dev_term.len()..].to_string();
            break
        }
    }

    if device_result.is_empty() {
        return Ok(None)
    }

    Ok(Some(device_result.to_string()))
}
