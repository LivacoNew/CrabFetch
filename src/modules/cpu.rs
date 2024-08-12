use core::str;
use std::{fs::{read_dir, File, ReadDir}, io::{BufRead, BufReader, Read}, path::{Component, Path}};

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, util::{self, is_flag_set_u32}, ModuleError};

pub struct CPUInfo {
    name: String,
    cores: u16,
    threads: u16,
    current_clock_mhz: f32,
    max_clock_mhz: f32,
    arch: String
}
#[derive(Deserialize)]
pub struct CPUConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub decimal_places: Option<u32>,
    pub remove_trailing_processor: bool
}

impl Module for CPUInfo {
    fn new() -> CPUInfo {
        CPUInfo {
            name: "Unknown".to_string(),
            cores: 0,
            threads: 0,
            current_clock_mhz: 0.0,
            max_clock_mhz: 0.0,
            arch: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.cpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.cpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.cpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.cpu.separator.as_ref().unwrap_or(&config.separator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.cpu.title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.cpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.cpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.cpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.cpu.separator.as_ref().unwrap_or(&config.separator);
        
        Self::default_style(config, max_title_size, &config.cpu.title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let dec_places: u32 = config.cpu.decimal_places.unwrap_or(config.decimal_places);

        config.cpu.format.replace("{name}", &self.name)
            .replace("{core_count}", &self.cores.to_string())
            .replace("{thread_count}", &self.threads.to_string())
            .replace("{current_clock_mhz}", &formatter::round(self.current_clock_mhz as f64, dec_places).to_string())
            .replace("{current_clock_ghz}", &formatter::round((self.current_clock_mhz / 1000.0) as f64, dec_places).to_string())
            .replace("{max_clock_mhz}", &formatter::round(self.max_clock_mhz as f64, dec_places).to_string())
            .replace("{max_clock_ghz}", &formatter::round((self.max_clock_mhz / 1000.0) as f64, dec_places).to_string())
            .replace("{arch}", &self.arch.to_string())
    }

    fn gen_info_flags(format: &str) -> u32 {
        // Figure out the info we need to fetch
        let mut info_flags: u32 = 0;

        if format.contains("{name}") {
            info_flags |= CPU_INFOFLAG_MODEL_NAME
        }
        if format.contains("{core_count}") {
            info_flags |= CPU_INFOFLAG_CORES
        }
        if format.contains("{thread_count}") {
            info_flags |= CPU_INFOFLAG_THREADS
        }
        if format.contains("{current_clock_mhz}") || format.contains("{current_clock_ghz}") {
            info_flags |= CPU_INFOFLAG_CURRENT_CLOCK
        }
        if format.contains("{max_clock_mhz}") || format.contains("{max_clock_ghz}") {
            info_flags |= CPU_INFOFLAG_MAX_CLOCK
        }
        if format.contains("{arch}") || format.contains("{arch}") {
            info_flags |= CPU_INFOFLAG_ARCH
        }

        info_flags
    }
}

const CPU_INFOFLAG_MODEL_NAME: u32 = 1;
const CPU_INFOFLAG_CORES: u32 = 2;
const CPU_INFOFLAG_THREADS: u32 = 4;
const CPU_INFOFLAG_CURRENT_CLOCK: u32 = 8;
const CPU_INFOFLAG_MAX_CLOCK: u32 = 16;
const CPU_INFOFLAG_ARCH: u32 = 32;

pub fn get_cpu(config: &Configuration) -> Result<CPUInfo, ModuleError> {
    let mut cpu: CPUInfo = CPUInfo::new();
    let info_flags: u32 = CPUInfo::gen_info_flags(&config.cpu.format);

    // This ones split into 2 as theres a lot to parse
    match get_basic_info(&mut cpu, info_flags) {
        Ok(_) => {},
        Err(e) => return Err(e)
    };
    match get_max_clock(&mut cpu, info_flags) {
        Ok(_) => {},
        Err(e) => return Err(e)
    };

    if config.cpu.remove_trailing_processor {
        // Tried doing this with Regex but it added 400 micro secs so fuck that shit
        let loc: usize = match cpu.name.find("-Core Processor") {
            Some(r) => r,
            None => return Ok(cpu), // ignore it
        };

        // Find the last space
        let search: &str = &cpu.name[..loc];
        let mut space_index: usize = 0;
        for (i, x) in search.chars().enumerate() {
            if x == ' ' && space_index < i {
                space_index = i;
            }
        }

        let replace_me: &str = &cpu.name[space_index..];
        cpu.name = cpu.name.replace(replace_me, "");
    }

    Ok(cpu)
}

fn get_basic_info(cpu: &mut CPUInfo, info_flags: u32) -> Result<(), ModuleError> {
    // Starts by reading and parsing /proc/cpuinfo
    // This gives us the cpu name, cores, threads and current clock
    let file: File = match File::open("/proc/cpuinfo") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /proc/cpuinfo - {}", e))),
    };

    let buffer: BufReader<File> = BufReader::new(file);
    let mut cpu_mhz_count: u8 = 0;
    let mut first_entry: bool = true;
    let mut cores: u16 = 0; // This acts as a backup for the "cpu cores" being missing
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line = line.unwrap();
        if line.is_empty() {
            first_entry = false;
            cores += 1;
        }

        if first_entry {
            if line.starts_with("model name") && is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
                cpu.name = line.split(": ").collect::<Vec<&str>>()[1].to_string();
            }
            if line.starts_with("cpu cores") && is_flag_set_u32(info_flags, CPU_INFOFLAG_CORES) {
                cpu.cores = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse cpu cores: {}", e))),
                }
            }
            if line.starts_with("siblings") && is_flag_set_u32(info_flags, CPU_INFOFLAG_THREADS) {
                cpu.threads = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse cpu threads: {}", e))),
                }
            }
            if line.starts_with("flags") && is_flag_set_u32(info_flags, CPU_INFOFLAG_ARCH) {
                // https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/include/asm/cpufeatures.h
                for flag in line.split(": ").collect::<Vec<&str>>()[1].split(' ') {
                    // prepare for trouble
                    cpu.arch = match flag {
                        "ia86" => {"IA86".to_string()}
                        "arch_capabilities" => {"IA32".to_string()}
                        "lm" => {"x86_64".to_string()}
                        _ => {continue} 
                    };
                    if !cpu.arch.is_empty() {
                        break;
                    }
                }
            }
        }
        if line.starts_with("cpu MHz") && is_flag_set_u32(info_flags, CPU_INFOFLAG_CURRENT_CLOCK) {
            cpu.current_clock_mhz += match line.split(": ").collect::<Vec<&str>>()[1].parse::<f32>() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse current cpu frequency: {}", e))),
            };
            cpu_mhz_count += 1;
        }
    }
    if cpu.cores == 0 && is_flag_set_u32(info_flags, CPU_INFOFLAG_CORES) {
        cpu.cores = cores;
        // Backup to /sys/devices/system/cpu/present for threads too
        // Thanks to https://stackoverflow.com/a/30150409
        let mut file: File = match File::open("/sys/devices/system/cpu/present") {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu/present - {}", e))),
        };
        let mut contents: String = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu/present - {}", e))),
        }
        cpu.threads = match contents.trim().split('-').last().unwrap().parse::<u16>() {
            Ok(r) => r + 1,
            Err(e) => return Err(ModuleError::new("CPU", format!("Failed to parse thread count from /sys/devices/system/cpu/present - {}", e))),
        };
    }

    // Android 
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        // This property was a fucking nightmare to find, only being able to find out about it here https://github.com/ArrowOS-Devices/android_device_xiaomi_daisy/blob/arrow-12.1/vendor.prop#L185
        // I have no idea how standard that property is, especially since I couldn't find it after
        // a couple hours of scowering Android's docs but oh well
        let props = AndroidSystemProperties::new();
        // https://github.com/termux/termux-api/issues/448#issuecomment-927345222
        if let Some(soc_manu) = props.get("ro.soc.manufacturer") {
            // chaining these let statements is only in the unstable branch so egypt it is
            if let Some(soc_model) = props.get("ro.soc.model") {
                cpu.name = format!("{} {}", soc_manu, soc_model);
            }
        }
    }

    cpu.current_clock_mhz /= cpu_mhz_count as f32;
    Ok(())
}
fn get_max_clock(cpu: &mut CPUInfo, info_flags: u32) -> Result<(), ModuleError> {
    if !is_flag_set_u32(info_flags, CPU_INFOFLAG_MAX_CLOCK) {
        return Ok(())
    }
    // All of this is relative to /sys/devices/system/cpu/X/cpufreq
    // There's 3 possible places to get the frequency in here;
    // - bios_limit - Only present if a limit is set in BIOS
    // - scaling_max_freq - The max freq set by the policy
    // - cpuinfo_max_freq - The max possible the CPU can run at uncapped
    //
    // This just takes the first of those three that are present
    //
    // Source: https://docs.kernel.org/admin-guide/pm/cpufreq.html

    let freq_path: Option<&Path> = util::find_first_path_exists(vec![
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/bios_limit"),
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"),
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
    ]);

    if freq_path.is_none() {
        // Back up to the repoted value in /proc/cpuinfo
        // At this point all I can assume is your in a VM, which doesn't have any of the above
        // paths and seems to keep a steady CPU frequency in here instead
        // Not the most elegant thing but I can't seem to find anything else to do
        cpu.max_clock_mhz = cpu.current_clock_mhz;
        return Ok(())
    }
    let freq_path: &Path = freq_path.unwrap();
    let mut freq_path_str: String = String::new();
    // sheesh
    for comp in freq_path.components().rev().take(2).collect::<Vec<Component<>>>().iter().rev() {
        freq_path_str.push('/');
        freq_path_str.push_str(comp.as_os_str().to_str().unwrap());
    }
    let freq_path: &str = &freq_path_str[1..];
    
    let dir: ReadDir = match read_dir("/sys/devices/system/cpu/") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu - {}", e)))
    };
    for entry in dir {
        let freq_path = match entry {
            Ok(r) => {
                let file_name = r.file_name();
                let file_name = file_name.to_str().unwrap();
                if !file_name.starts_with("cpu") || file_name == "cpuidle" || file_name.starts_with("cpufreq") {
                    continue
                }
                r.path().join(freq_path)
            },
            Err(_) => continue, // ?
        };
        let freq_path = freq_path.as_path();

        match util::file_read(freq_path) {
            Ok(r) => {
                match r.trim().parse::<f32>() {
                    Ok(r) => cpu.max_clock_mhz = f32::max(r / 1000.0, cpu.max_clock_mhz),
                    Err(e) => return Err(ModuleError::new("CPU", format!("Unable to parse f32 from {} - {}", freq_path.to_str().unwrap(), e)))
                };
            },
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from {} - {}", freq_path.to_str().unwrap(), e))),
        };
    }

    Ok(())
}
