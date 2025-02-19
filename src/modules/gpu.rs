use core::str;
use std::{fs::{self, DirEntry, File, ReadDir}, io::{BufRead, BufReader}, path::Path};

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
    pub detect_through_driver: bool,

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
            .replace("{vram}", &formatter::auto_format_bytes(u64::from(self.vram_mb * 1000), use_ibis, 0))
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
            info_flags |= GPU_INFOFLAG_VRAM;
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

    if config.gpu.detect_through_driver {
        match fill_from_drivers(&mut gpus, config.gpu.amd_accuracy, config.gpu.ignore_disabled_gpus, info_flags) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
    } else {
        match fill_from_pcisysfile(&mut gpus, config.gpu.amd_accuracy, config.gpu.ignore_disabled_gpus, info_flags) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
    }

    Ok(gpus)
}

// TODO These need audited, I can only test AMD so have no clue if NVIDIA/Intel are correct or not?
const KNOWN_GPU_DRIVERS: &[&str] = &[
    // AMD
    "amdgpu",
    "radeon",
    // NVIDIA
    "nvidia",
    "nouveau",
    // Intel
    "i915",
    "xe"
];
fn fill_from_drivers(gpus: &mut Vec<GPUInfo>, amd_accuracy: bool, ignore_disabled: bool, info_flags: u32) -> Result<(), ModuleError> {
    let dir: ReadDir = match fs::read_dir("/sys/bus/pci/drivers") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from /sys/bus/pci/drivers: {e}")))
    };

    for driver_dir in dir {
        let Ok(d) = driver_dir else {
            continue;
        };

        // Check if it's a known GPU driver.
        if !KNOWN_GPU_DRIVERS.contains(&d.path().file_name().unwrap().to_str().unwrap()) {
            continue;
        }

        // Go inside and scan all devices being driven by it 
        let driver_dir: ReadDir = match fs::read_dir(d.path()) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from {:?}: {e}", d.path())))
        };

        for device in driver_dir {
            let Ok(dev) = device else {
                continue;
            };

            // Is it an device?
            // AKA is it a dir and is it not called "module"?
            if !dev.path().is_dir() {
                continue;
            }
            if dev.path().file_name().unwrap().to_str().unwrap() == "module" {
                continue;
            }

            // So, in *theory*, this is a GPU 
            // Let's scan it :)
            let Ok(Some(gpu)) = scan_pci_dir(&dev, amd_accuracy, ignore_disabled, info_flags) else {
                continue;
            };
            gpus.push(gpu);
        }
    }

    Ok(())
}

fn fill_from_pcisysfile(gpus: &mut Vec<GPUInfo>, amd_accuracy: bool, ignore_disabled: bool, info_flags: u32) -> Result<(), ModuleError> {
    let dir: ReadDir = match fs::read_dir("/sys/bus/pci/devices") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from /sys/bus/pci/devices: {e}"))),
    };
    for dev_dir in dir {
        // This does the following;
        // Reads the modalias file, letting us have access to the class/vendor/device all at once
        // Then checks if the class is 0x03 (https://github.com/torvalds/linux/blob/master/include/linux/pci_ids.h#L38)
        // Then we know it's a GPU and can go probing for all the rest of the info
        let Ok(d) = dev_dir else {
            continue;
        };

        let Ok(Some(gpu)) = scan_pci_dir(&d, amd_accuracy, ignore_disabled, info_flags) else {
            continue;
        };
        gpus.push(gpu);
    }

    Ok(())
}


fn scan_pci_dir(d: &DirEntry, amd_accuracy: bool, ignore_disabled: bool, info_flags: u32) -> Result<Option<GPUInfo>, ModuleError> {
    // Credit: https://wiki.archlinux.org/title/Modalias
    // v - Vendor ID
    // d - Device ID
    // sv/sd - Subsys Vendor/Device, not needed for us (although could posssibly identify gpu vendor info?) 
    // bc - Base Class 
    // sc - Sub-class, not needed for us
    // i - Programming interface, again useless for us
    // From what I can see, these are all in a fixed length so we can just use substringing to
    // find our info opposed to any fancy parsing shit
    // An example from my GPU: v00001002d0000747Esv00001DA2sd0000D475bc03sc00i00
    let Ok(modalias) = util::file_read(&d.path().join("modalias")) else {
        return Ok(None);
    };
    let modalias_contents: String = modalias[4..].to_string(); // strip the prefixing "pci:"

    // Check the class first things first, checking it's a display device 
    // Only checking the base class for it being 0x03 
    // v00001002d0000747Esv00001DA2sd0000D475bc03sc00i00
    let base_class: &str = &modalias_contents[40..42];
    // Not a display device
    // And yes, I'm doing this check with a string instead of parsing it with an AND, fuck you.
    if base_class != "03" {
        return Ok(None);
    }

    // Next, check the GPU isn't disabled
    if ignore_disabled {
        let Ok(enabled_str) = util::file_read(&d.path().join("enable")) else {
            return Ok(None);
        };
        if enabled_str.trim() == "0" {
            return Ok(None);
        }
    }

    // Device info time 
    let mut gpu: GPUInfo = GPUInfo::new();
    // Vendor/Device
    if is_flag_set_u32(info_flags, GPU_INFOFLAG_MODEL) || is_flag_set_u32(info_flags, GPU_INFOFLAG_VENDOR) {
        let vendor_id: String = modalias_contents[5..9].to_string();
        let device_id: String = modalias_contents[14..18].to_string();

        // Being more accurate with AMD
        if vendor_id == "1002" && amd_accuracy {
            gpu.vendor = String::from("Advanced Micro Devices, Inc. [AMD/ATI]");
            let revision_id: String = match util::file_read(&d.path().join("revision")) {
                Ok(r) => r[2..].trim().to_string(),
                Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from file: {e}"))),
            };
            if let Some(r) = search_amd_model(&device_id, &revision_id)? {
                gpu.model = r;
            }
        }
        if gpu.model == "Unknown" {
            (gpu.vendor, gpu.model) = search_pci_ids(&vendor_id, &device_id)?;
        }
    }

    // Finally, Vram
    if is_flag_set_u32(info_flags, GPU_INFOFLAG_VRAM) {
        if let Ok(r) = util::file_read(&d.path().join("mem_info_vram_total")) {
            gpu.vram_mb = match u32::try_from(r.trim().parse::<u64>().unwrap() / 1024 / 1024) {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("GPU", format!("Failed to convert vram to u32: {e}"))),
            }
        }
    }

    Ok(Some(gpu))
}

// This uses pci.ids inside pciutils to identify devices:
// https://man.archlinux.org/man/core/pciutils/pci.ids.5.en
//
// To prevent having to relicence to GPL I don't distribute a copy of this, and simply have to
// rely on the host's copy. Problem then becomes that different distros seem to place this file
// in different places
// I'll try to find it in as many places as possible but ultimately can't cover every place. If
// you know the places, make a PR/Issue and i'll add it in. Fucking hate licences that work
// like this but oh well.
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
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from {} - {e}", ids_path.display()))),
    };
    let buffer: BufReader<File> = BufReader::new(file);

    // parsing this file is weird
    let mut vendor_result: String = String::new();
    let mut device_result: String = String::new();
    // Find the vendor ID + device in the list
    let vendor_term: String = String::from(vendor).to_lowercase();
    let dev_term: String = (String::from('\t') + device).to_lowercase();
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
        Err(e) => return Err(ModuleError::new("GPU", format!("Can't read from {} - {e}", ids_path.display()))),
    };
    let buffer: BufReader<File> = BufReader::new(file);

    let mut device_result: String = String::new();
    let dev_term: String = format!("{device},\t{revision},\t").to_lowercase().to_string();
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

mod tests {
    #[test]
    fn test_pci_ids_normal() {
        use crate::{module::ModuleError, modules::gpu::search_pci_ids};
        use std::collections::HashMap;

        let tests: HashMap<(&str, &str), (&str, &str)> = HashMap::from([
            // AMD
            (("1002", "747e"), ("Advanced Micro Devices, Inc. [AMD/ATI]", "Navi 32 [Radeon RX 7700 XT / 7800 XT]")),
            (("1002", "73ff"), ("Advanced Micro Devices, Inc. [AMD/ATI]", "Navi 23 [Radeon RX 6600/6600 XT/6600M]")),
            // NOTE: Commented out since the githubs actions env is too outdated
            // (("1002", "744c"), ("Advanced Micro Devices, Inc. [AMD/ATI]", "Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]")),

            // NVIDIA
            (("10DE", "2203"), ("NVIDIA Corporation", "GA102 [GeForce RTX 3090 Ti]")),
            (("10DE", "2184"), ("NVIDIA Corporation", "TU116 [GeForce GTX 1660]")),

            // Intel
            (("8086", "5690"), ("Intel Corporation", "DG2 [Arc A770M]")),
            (("8086", "e20b"), ("Intel Corporation", "Battlemage G21 [Arc B580]"))
        ]);

        for test in tests {
            let result: Result<(String, String), ModuleError> = search_pci_ids(test.0.0, test.0.1);
            assert!(result.is_ok());
            let result: (String, String) = result.unwrap();
            assert_eq!(result.0, test.1.0);
            assert_eq!(result.1, test.1.1);
        }
    }

    // AMD Accuracy
    #[test]
    fn test_pci_ids_amd() {
        use crate::{module::ModuleError, modules::gpu::search_amd_model};
        use std::collections::HashMap;

        let tests: HashMap<(&str, &str), &str> = HashMap::from([
            (("747e", "C8"), "AMD Radeon RX 7800 XT"),
            (("73ff", "C1"), "AMD Radeon RX 6600 XT"),
            (("744c", "c8"), "AMD Radeon RX 7900 XTX"),
        ]);

        for test in tests {
            let result: Result<Option<String>, ModuleError> = search_amd_model(test.0.0, test.0.1);
            assert!(result.is_ok());
            let result: Option<String> = result.unwrap();
            assert!(result.is_some());
            let result: String = result.unwrap();
            assert_eq!(result, test.1);
        }
    }
}
